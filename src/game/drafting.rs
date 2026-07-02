use dioxus::prelude::*;
use jiff::{SignedDuration, Zoned};
use rand::{distr::Uniform, rngs::SmallRng, RngExt};
use serde::Deserialize;
use titlecase::Titlecase;

use crate::{
    game::{Game, GameStoreExt},
    ui::number_format::F64,
    ART_ASSET_FOLDER,
};

static DECK_DEFINITIONS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/decks.toml"));

#[derive(Clone, Store)]
pub struct Drafting {
    pub decks: Vec<Deck>,

    pub current_timestamp: Zoned,
    pub rng: SmallRng,

    pub inventory: Vec<Card>,

    pub selected_deck: usize,
    pub last_draft_timestamp: Zoned,
    pub draft_cooldown: SignedDuration,
    pub can_draft: bool,
}

#[derive(Clone, Store)]
pub struct Deck {
    pub name: String,
    pub description: String,
    pub initial_card_level: u8,

    pub cards: Vec<CardTemplate>,
}

#[derive(Clone, Store)]
pub struct Card {
    pub template: CardTemplate,

    pub level: u8,
}

#[derive(Clone, Store)]
pub struct CardTemplate {
    pub name: String,
    pub description: String,
    pub image_path: String,
    #[expect(dead_code)]
    pub rarity: u8,
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
}

impl Drafting {
    pub fn new_game() -> Self {
        let deck_definitions: DeckDefinitions = toml::from_str(DECK_DEFINITIONS).unwrap();
        let decks: Vec<Deck> = deck_definitions.decks.into_iter().map(Into::into).collect();
        let current_timestamp = Zoned::now();
        // let draft_cooldown = SignedDuration::from_secs(5);
        let draft_cooldown = SignedDuration::from_millis(1);

        Self {
            decks,

            current_timestamp: current_timestamp.clone(),
            rng: rand::make_rng(),

            inventory: Vec::new(),

            selected_deck: 0,
            last_draft_timestamp: current_timestamp - draft_cooldown,
            draft_cooldown,
            can_draft: true,
        }
    }
}

impl From<DeckDefinition> for Deck {
    fn from(deck_definition: DeckDefinition) -> Self {
        let cards: Vec<CardTemplate> = deck_definition.cards.into_iter().map(Into::into).collect();
        Self {
            name: deck_definition.name,
            description: deck_definition.description,

            initial_card_level: deck_definition.initial_card_level,

            cards,
        }
    }
}

impl From<CardDefinition> for CardTemplate {
    fn from(
        CardDefinition {
            name,
            description,
            image,
            rarity,
        }: CardDefinition,
    ) -> Self {
        Self {
            name: name.titlecase(),
            description,
            image_path: format!(
                "{ART_ASSET_FOLDER}/{}",
                image.as_deref().unwrap_or("missing.png"),
            ),
            rarity,
        }
    }
}

#[store(pub)]
impl<Lens> Store<Drafting, Lens> {
    fn do_update(&mut self) {
        let current_timestamp = Zoned::now();
        self.current_timestamp().set(current_timestamp.clone());

        let can_draft = current_timestamp
            >= &*self.last_draft_timestamp().read() + *self.draft_cooldown().read();
        self.can_draft().set(can_draft);
    }

    fn do_rebirth(&mut self) {}

    fn draft(&mut self) {
        if !*self.can_draft().read() {
            return;
        }

        let current_timestamp = self.current_timestamp().read().clone();
        self.last_draft_timestamp().set(current_timestamp);

        let selected_deck = *self.selected_deck().read();
        let card = self.decks().get(selected_deck).unwrap().draft(self.rng());
        self.inventory().push(card);
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

#[store(pub)]
impl<Lens> Store<Deck, Lens> {
    fn draft<RngLens: Copy + Writable<Target = SmallRng>>(
        &self,
        rng: Store<SmallRng, RngLens>,
    ) -> Card {
        let mut rng = rng;
        let card_count = self.cards().len();
        let card_index = rng.with_mut(|rng| {
            let distribution = Uniform::new(0, card_count).unwrap();
            rng.sample(distribution)
        });

        let template = self.cards().get(card_index).unwrap().read().clone();
        let level = *self.initial_card_level().read();

        Card { template, level }
    }
}

type MappedCardProperty<Type, Lens> = Store<
    Type,
    MappedMutSignal<
        Type,
        MappedMutSignal<
            CardTemplate,
            Lens,
            for<'a> fn(&'a Card) -> &'a CardTemplate,
            for<'a> fn(&'a mut Card) -> &'a mut CardTemplate,
        >,
        for<'a> fn(&'a CardTemplate) -> &'a Type,
        for<'a> fn(&'a mut CardTemplate) -> &'a mut Type,
    >,
>;

#[store(pub)]
impl<Lens> Store<Card, Lens> {
    fn name(&self) -> MappedCardProperty<String, Lens> {
        self.template().name()
    }

    fn description(&self) -> MappedCardProperty<String, Lens> {
        self.template().description()
    }

    fn image_path(&self) -> MappedCardProperty<String, Lens> {
        self.template().image_path()
    }
}

#[component]
pub fn DraftingView() -> Element {
    let game = use_context::<Store<Game>>();
    let mut drafting = game.drafting();

    let drafting_disabled = use_memo(move || !*drafting.can_draft().read());
    let deck_description = use_memo(move || {
        let selected_deck = *drafting.selected_deck().read();
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
            div { class: "horizontal-multirow",
                for card in drafting.inventory().iter() {
                    CardView { card }
                }
            }
        }
    }
}

#[component]
pub fn CardView(card: ReadStore<Card>) -> Element {
    rsx! {
        div { class: "card",
            span { class: "card-title", "{card.template().name()} Level\u{00A0}{card.level()}" }
            img { class: "card-image", src: "{card.image_path()}" }
            p { class: "card-description", {card.template().description()} }
        }
    }
}
