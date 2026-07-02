use std::ops::{Range, RangeInclusive};

use dioxus::prelude::*;
use jiff::{SignedDuration, Zoned};
use rand::{distr::weighted::WeightedIndex, rngs::SmallRng, RngExt};
use serde::Deserialize;
use titlecase::Titlecase;

use crate::{
    game::{Game, GameStoreExt},
    ui::number_format::F64,
    ART_ASSET_FOLDER,
};

static DECK_DEFINITIONS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/decks.toml"));
static RARITIES: &[&str] = &[
    "Trash",
    "Common",
    "Uncommon",
    "Rare",
    "Epic",
    "Legendary",
    "Mythic",
    "Divine",
];

#[derive(Clone, Store)]
pub struct Drafting {
    pub cards: Vec<Card>,
    pub decks: Vec<Deck>,
    pub rarity_weight_table: Vec<f64>,

    pub current_timestamp: Zoned,
    pub rng: SmallRng,

    pub selected_deck: Option<usize>,
    pub last_draft_timestamp: Zoned,
    pub just_drafted: Option<usize>,
    pub draft_cooldown: SignedDuration,
    pub can_draft: bool,

    pub attack_factor: f64,
    pub defense_factor: f64,
    pub hitpoints_factor: f64,
}

#[derive(Clone, Store)]
pub struct Deck {
    #[expect(dead_code)]
    pub deck_index: usize,
    pub name: String,
    pub description: String,

    pub initial_card_level: u8,

    pub cards: Range<usize>,
    #[expect(dead_code)]
    pub rarities: RangeInclusive<u8>,
}

#[derive(Clone, Store)]
pub struct Card {
    #[expect(dead_code)]
    pub card_index: usize,
    pub name: String,
    pub description: String,
    pub image: String,
    pub rarity: u8,

    /// Level of zero represents a not yet found card.
    /// Level 100 is max level.
    pub level: u8,
    pub just_drafted: bool,

    pub effect: CardEffect,
}

#[derive(Clone)]
pub enum CardEffect {
    None,
    /// Multiply the weight of the affected rarity by the factor.
    RarityFactor {
        affected_rarity: u8,
        factor: f64,
    },
    AttackFactor {
        factor: f64,
    },
    DefenseFactor {
        factor: f64,
    },
    HitpointsFactor {
        factor: f64,
    },
}

#[derive(Deserialize)]
struct DeckDefinitions {
    #[serde(alias = "deck")]
    decks: Vec<DeckDefinition>,
}

#[derive(Deserialize)]
struct DeckDefinition {
    name: String,
    description: String,

    initial_card_level: u8,

    #[serde(alias = "card")]
    cards: Vec<CardDefinition>,
}

#[derive(Deserialize)]
struct CardDefinition {
    name: String,
    description: String,
    image: Option<String>,
    rarity: u8,

    affected_rarity: Option<u8>,
    rarity_factor: Option<f64>,

    attack_factor: Option<f64>,
    defense_factor: Option<f64>,
    hitpoints_factor: Option<f64>,
}

impl Drafting {
    pub fn new_game() -> Self {
        let rarity_weight_table: Vec<_> = (0..RARITIES.len() as i32)
            .map(|i| 10_f64.powi(i))
            .rev()
            .collect();

        let deck_definitions: DeckDefinitions = toml::from_str(DECK_DEFINITIONS).unwrap();

        let mut decks = Vec::new();
        let mut cards = Vec::new();
        for DeckDefinition {
            name,
            description,
            initial_card_level,
            cards: deck_cards,
        } in deck_definitions.decks
        {
            let deck_index = decks.len();

            let card_offset = cards.len();
            let mut min_rarity = u8::MAX;
            let mut max_rarity = u8::MIN;
            for CardDefinition {
                name,
                description,
                image,
                rarity,

                affected_rarity,
                rarity_factor,

                attack_factor,
                defense_factor,
                hitpoints_factor,
            } in deck_cards
            {
                let card_index = cards.len();
                min_rarity = min_rarity.min(rarity);
                max_rarity = max_rarity.max(rarity);

                debug_assert!(
                    [
                        affected_rarity.is_some(),
                        attack_factor.is_some(),
                        defense_factor.is_some(),
                        hitpoints_factor.is_some()
                    ]
                    .into_iter()
                    .filter(|&x| x)
                    .count()
                        <= 1
                );

                let effect = if let Some(affected_rarity) = affected_rarity {
                    if let Some(factor) = rarity_factor {
                        CardEffect::RarityFactor {
                            affected_rarity,
                            factor,
                        }
                    } else {
                        debug_assert!(false);
                        CardEffect::None
                    }
                } else if let Some(factor) = attack_factor {
                    CardEffect::AttackFactor { factor }
                } else if let Some(factor) = defense_factor {
                    CardEffect::DefenseFactor { factor }
                } else if let Some(factor) = hitpoints_factor {
                    CardEffect::HitpointsFactor { factor }
                } else {
                    CardEffect::None
                };

                cards.push(Card {
                    name: name.titlecase(),
                    description,
                    image: image.unwrap_or_else(|| "missing.png".to_string()),
                    rarity,
                    card_index,
                    level: 0,
                    just_drafted: false,
                    effect,
                });
            }
            let card_limit = cards.len();

            decks.push(Deck {
                name: name.titlecase(),
                description,
                deck_index,
                initial_card_level,
                cards: card_offset..card_limit,
                rarities: min_rarity..=max_rarity,
            });
        }

        let current_timestamp = Zoned::now();
        // let draft_cooldown = SignedDuration::from_secs(5);
        let draft_cooldown = SignedDuration::from_millis(1);

        Self {
            decks,
            cards,
            rarity_weight_table,

            current_timestamp: current_timestamp.clone(),
            rng: rand::make_rng(),

            selected_deck: None,
            last_draft_timestamp: current_timestamp - draft_cooldown,
            just_drafted: None,
            draft_cooldown,
            can_draft: true,

            attack_factor: 1.0,
            defense_factor: 1.0,
            hitpoints_factor: 1.0,
        }
    }
}

impl CardEffect {
    fn scale_by_level(&self, level: u8) -> Self {
        match self {
            CardEffect::None => CardEffect::None,
            CardEffect::RarityFactor {
                affected_rarity,
                factor,
            } => CardEffect::RarityFactor {
                affected_rarity: *affected_rarity,
                factor: factor.powf(level as f64 / 100.0),
            },
            CardEffect::AttackFactor { factor } => CardEffect::AttackFactor {
                factor: factor.powf(level as f64 / 100.0),
            },
            CardEffect::DefenseFactor { factor } => CardEffect::DefenseFactor {
                factor: factor.powf(level as f64 / 100.0),
            },
            CardEffect::HitpointsFactor { factor } => CardEffect::HitpointsFactor {
                factor: factor.powf(level as f64 / 100.0),
            },
        }
    }
}

#[store(pub)]
impl<Lens> Store<Drafting, Lens> {
    fn do_update(&mut self) {
        let current_timestamp = Zoned::now();
        self.current_timestamp().set(current_timestamp.clone());

        let can_draft = (current_timestamp
            >= &*self.last_draft_timestamp().read() + *self.draft_cooldown().read())
            && self.selected_deck().read().is_some();
        self.can_draft().set(can_draft);
    }

    fn do_rebirth(&mut self) {
        self.can_draft().set(false);
        self.selected_deck().set(None);
        self.just_drafted().set(None);
    }

    fn draft(&mut self) {
        if !*self.can_draft().read() {
            return;
        }
        let Some(selected_deck) = *self.selected_deck().read() else {
            debug_assert!(false);
            return;
        };

        self.recompute_card_effects();

        let current_timestamp = self.current_timestamp().read().clone();
        self.last_draft_timestamp().set(current_timestamp);

        let card_range = self
            .decks()
            .get(selected_deck)
            .unwrap()
            .cards()
            .read()
            .clone();
        let rarity_weight_table = self.rarity_weight_table().read().clone();
        let weighted_index = WeightedIndex::new(
            self.cards()
                .iter()
                .take(card_range.end)
                .skip(card_range.start)
                .map(|card| rarity_weight_table[*card.rarity().read() as usize]),
        )
        .unwrap();

        let card_index = self.rng().with_mut(move |rng| rng.sample(&weighted_index));
        let card_index = card_index + card_range.start;

        let level_increment = *self
            .decks()
            .get(selected_deck)
            .unwrap()
            .initial_card_level()
            .read();
        self.cards()
            .get(card_index)
            .unwrap()
            .level()
            .with_mut(|level| {
                *level = level.saturating_add(level_increment).min(100);
            });

        let just_drafted = *self.just_drafted().read();
        if let Some(just_drafted) = just_drafted {
            self.cards()
                .get(just_drafted)
                .unwrap()
                .just_drafted()
                .set(false);
        }
        self.cards()
            .get(card_index)
            .unwrap()
            .just_drafted()
            .set(true);
        self.just_drafted().set(Some(card_index));

        self.recompute_card_effects();
    }

    fn recompute_card_effects(&mut self) {
        let mut rarity_weight_table: Vec<_> = (0..RARITIES.len() as i32)
            .map(|i| 10_f64.powi(i))
            .rev()
            .collect();
        let mut attack_factor = 1.0;
        let mut defense_factor = 1.0;
        let mut hitpoints_factor = 1.0;

        for card in self.cards().iter() {
            let level = *card.level().read();
            let effect = card.effect().read().scale_by_level(level);

            match effect {
                CardEffect::None => {}
                CardEffect::RarityFactor {
                    affected_rarity,
                    factor,
                } => rarity_weight_table[affected_rarity as usize] *= factor,
                CardEffect::AttackFactor { factor } => attack_factor *= factor,
                CardEffect::DefenseFactor { factor } => defense_factor *= factor,
                CardEffect::HitpointsFactor { factor } => hitpoints_factor *= factor,
            }
        }

        self.rarity_weight_table().set(rarity_weight_table);
        self.attack_factor().set(attack_factor);
        self.defense_factor().set(defense_factor);
        self.hitpoints_factor().set(hitpoints_factor);
    }

    fn remaining_cooldown(&self) -> Option<SignedDuration> {
        let current_timestamp = self.current_timestamp().read().clone();
        let last_draft_timestamp = self.last_draft_timestamp().read().clone();
        let draft_cooldown = *self.draft_cooldown().read();

        let elapsed = SignedDuration::try_from(current_timestamp - last_draft_timestamp)
            .unwrap_or(draft_cooldown);
        let remaining = draft_cooldown - elapsed;
        if remaining > SignedDuration::from_secs(0) {
            Some(remaining)
        } else {
            None
        }
    }

    fn remaining_cooldown_seconds(&self) -> Option<f64> {
        self.remaining_cooldown()
            .map(|remaining| remaining.as_secs_f64())
    }
}

#[component]
pub fn DraftingView() -> Element {
    let game = use_context::<Store<Game>>();
    let mut drafting = game.drafting();

    let drafting_disabled = use_memo(move || !*drafting.can_draft().read());
    let deck_description = use_memo(move || {
        let Some(selected_deck) = *drafting.selected_deck().read() else {
            return "Select a deck".to_string();
        };
        drafting
            .decks()
            .get(selected_deck)
            .unwrap()
            .description()
            .read()
            .clone()
    });

    rsx! {
        div { class: "vertical", style: "padding: 10px;",
            h2 { "Drafting" }
            div { class: "horizontal",
                label { "Deck:" }
                select {
                    onchange: move |event| {
                        if let Ok(value) = event.value().parse() {
                            drafting.selected_deck().set(Some(value));
                        } else {
                            debug_assert_eq!(event.value(), "none");
                            drafting.selected_deck().set(None);
                        }
                    },
                    option { value: "none", "None" }
                    for (id, deck) in drafting.decks().iter().enumerate() {
                        option { value: "{id}", "{deck.name()}" }
                    }
                }
            }
            span { "{deck_description}" }
            button {
                onclick: move |_| {
                    drafting.draft();
                },
                disabled: drafting_disabled,
                "Draft"
                if let Some(remaining_cooldown) = drafting.remaining_cooldown_seconds() {
                    " (Cooldown: "
                    F64 { number: remaining_cooldown }
                    " seconds)"
                } else {
                    ""
                }
            }
            RaritiesView {}
            InventoryView {}
        }
    }
}

#[component]
fn RaritiesView() -> Element {
    let game = use_context::<Store<Game>>();
    let drafting = game.drafting();
    let rarity_weight_table = drafting.rarity_weight_table();

    rsx! {
        table { class: "rarities",
            tr { class: "rarities",
                th { class: "rarities", "Rarity" }
                th { class: "rarities", "Weight" }
            }
            for (rarity, weight) in RARITIES.iter().zip(rarity_weight_table.read().iter()) {
                tr { class: "rarities",
                    td { class: "rarities", "{rarity}" }
                    td { class: "rarities",
                        F64 { number: *weight, format_as_integer: true }
                    }
                }
            }
        }

    }
}

#[component]
fn InventoryView() -> Element {
    let game = use_context::<Store<Game>>();
    let drafting = game.drafting();

    if let Some(selected_deck) = *drafting.selected_deck().read() {
        let card_range = drafting
            .decks()
            .get(selected_deck)
            .unwrap()
            .cards()
            .read()
            .clone();

        rsx! {
            div { class: "horizontal-multirow",
                for card in drafting.cards().iter().take(card_range.end).skip(card_range.start) {
                    CardView { card }
                }
            }
        }
    } else {
        rsx! {}
    }
}

#[component]
fn CardView(card: ReadStore<Card>) -> Element {
    let background_image = use_memo(move || {
        format!(
            "{ART_ASSET_FOLDER}/cards/rarity-{}.png",
            if *card.level().read() > 0 {
                format!("{}", *card.rarity().read())
            } else {
                "locked".to_string()
            },
        )
    });

    let style = use_memo(move || {
        let mut style = if *card.just_drafted().read() {
            "border-color: red;".to_string()
        } else {
            "border-color: black;".to_string()
        };

        style.push_str("background-image: url(");
        style.push_str(background_image.read().as_str());
        style.push_str("); background-size: cover;");

        style
    });

    let image_path = use_memo(move || {
        let image_path = card.image();
        let image_path = image_path.read();
        format!(
            "{ART_ASSET_FOLDER}/cards/{}",
            if *card.level().read() > 0 {
                image_path.as_str()
            } else {
                "locked.png"
            },
        )
    });

    let effect = use_memo(move || {
        let level = *card.level().read();
        match card.effect().read().scale_by_level(level) {
            CardEffect::None => rsx! {},
            CardEffect::RarityFactor {
                affected_rarity,
                factor,
            } => {
                if factor == 1.0 {
                    rsx! {}
                } else if factor > 1.0 {
                    rsx! {
                        "{RARITIES[affected_rarity as usize]} weight +"
                        F64 { number: (factor - 1.0) * 100.0 }
                        "%"
                    }
                } else {
                    rsx! {
                        "{RARITIES[affected_rarity as usize]} weight -"
                        F64 { number: (1.0 - factor) * 100.0 }
                        "%"
                    }
                }
            }
            CardEffect::AttackFactor { factor } => {
                rsx! {
                    "Attack times "
                    F64 { number: factor }
                }
            }
            CardEffect::DefenseFactor { factor } => {
                rsx! {
                    "Defense times "
                    F64 { number: factor }
                }
            }
            CardEffect::HitpointsFactor { factor } => {
                rsx! {
                    "Hitpoints times "
                    F64 { number: factor }
                }
            }
        }
    });

    if *card.level().read() > 0 {
        rsx! {
            div { class: "card", style,
                span { class: "card-title", "{card.name()} Level\u{00A0}{card.level()}" }
                img { class: "card-image", src: image_path }
                p { class: "card-effect", {effect} }
                p { class: "card-description", {card.description()} }
            }
        }
    } else {
        rsx! {
            div { class: "card", style,
                span { class: "card-title", "Locked" }
                img { class: "card-image", src: image_path }
                p { class: "card-effect" }
                p { class: "card-description" }
            }
        }
    }
}
