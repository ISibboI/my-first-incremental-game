use std::sync::LazyLock;

use dioxus::prelude::*;
use dioxus_stores::index::IndexWrite;
use regex::Regex;
use serde::Deserialize;

use crate::{
    game::{
        drafting::{Drafting, DraftingStoreExt},
        rebirth::{Rebirth, RebirthStoreExt},
        training::{Training, TrainingStoreExt},
        Game, GameStoreExt,
    },
    ui::{number_format::F64, ProgressBar},
    TICKS_PER_SECOND,
};

const HITPOINTS_REGENERATION_TO_FULL_SECONDS: f64 = 120.0;
static BOSS_DEFINITIONS: &str =
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/data/bosses.toml"));

#[derive(Clone, Store)]
pub struct Bossfight {
    pub bosses: Vec<Boss>,

    pub current_boss: usize,
    pub current_boss_hitpoints: f64,
    pub current_player_hitpoints: f64,
    pub is_fighting: bool,
}

#[derive(Clone, Store)]
pub struct Boss {
    pub name: String,
    pub unlocks: Vec<String>,
    pub story: Vec<String>,
    pub attack: f64,
    pub defense: f64,
    pub max_hitpoints: f64,
}

#[derive(Deserialize)]
struct BossDefinitions {
    #[serde(alias = "boss")]
    bosses: Vec<BossDefinition>,
}
#[derive(Clone, Store, Deserialize)]
struct BossDefinition {
    name: String,
    unlocks: Option<String>,
    story: String,
    power: f64,
}

#[derive(Clone, Store)]
pub struct BossfightStats {
    pub attack: f64,
    pub defense: f64,
    pub hitpoints: f64,
}

impl Bossfight {
    pub fn new_game(bossfight_stats: &BossfightStats) -> Self {
        let boss_definitions: BossDefinitions = toml::from_str(BOSS_DEFINITIONS).unwrap();
        let bosses: Vec<Boss> = boss_definitions
            .bosses
            .into_iter()
            .map(Into::into)
            .collect();
        let current_boss_hitpoints = bosses.first().map_or(0.0, |boss| boss.max_hitpoints);

        Self {
            bosses,

            current_boss: 0,
            current_boss_hitpoints,
            current_player_hitpoints: bossfight_stats.hitpoints,
            is_fighting: false,
        }
    }
}

impl From<BossDefinition> for Boss {
    fn from(
        BossDefinition {
            name,
            unlocks,
            story,
            power,
        }: BossDefinition,
    ) -> Self {
        static STORY_LINEBREAK_REGEX: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\n\s*\n").unwrap());
        let story = STORY_LINEBREAK_REGEX
            .split(&story)
            .map(str::trim)
            .filter(|paragraph| !paragraph.is_empty())
            .map(ToString::to_string)
            .collect();
        let unlocks = STORY_LINEBREAK_REGEX
            .split(unlocks.as_deref().unwrap_or(""))
            .map(str::trim)
            .filter(|paragraph| !paragraph.is_empty())
            .map(ToString::to_string)
            .collect();

        Self {
            name,
            unlocks,
            story,
            attack: power * 2.0,
            defense: power,
            max_hitpoints: power * 1e3,
        }
    }
}

type MappedBossStore<Lens> = Store<
    Boss,
    IndexWrite<
        usize,
        MappedMutSignal<
            Vec<Boss>,
            Lens,
            fn(&Bossfight) -> &Vec<Boss>,
            fn(&mut Bossfight) -> &mut Vec<Boss>,
        >,
    >,
>;

#[store(pub)]
impl<Lens> Store<Bossfight, Lens> {
    fn do_update<BossfightStatsLens: Copy + Readable<Target = BossfightStats>>(
        &mut self,
        bossfight_stats: Store<BossfightStats, BossfightStatsLens>,
    ) {
        if *self.is_fighting().read() {
            // Deal damage to each other.
            let damage_dealt = (*bossfight_stats.attack().read()
                - self.boss().map_or(0.0, |boss| *boss.defense().read()))
            .max(0.0);
            let damage_taken = (self.boss().map_or(0.0, |boss| *boss.attack().read())
                - *bossfight_stats.defense().read())
            .max(0.0);

            self.current_boss_hitpoints().with_mut(|hp| {
                *hp -= damage_dealt;
                *hp = hp.max(0.0);
            });
            self.current_player_hitpoints().with_mut(|hp| {
                *hp -= damage_taken;
                *hp = hp.max(0.0);
            });

            // Check for Win and defeat conditions.
            if *self.current_boss_hitpoints().read() <= 0.0 {
                self.current_boss().with_mut(|boss| *boss += 1);
                self.current_boss_hitpoints()
                    .set(self.boss().map_or(0.0, |boss| *boss.max_hitpoints().read()));
                self.is_fighting().set(false);
            }

            if *self.current_player_hitpoints().read() <= 0.0 {
                self.is_fighting().set(false);
            }
        }

        // Automatically regenerate hitpoints over time.
        let max_boss_hitpoints = self.boss().map_or(0.0, |boss| *boss.max_hitpoints().read());
        let current_boss_hitpoints = (*self.current_boss_hitpoints().read()
            + max_boss_hitpoints / HITPOINTS_REGENERATION_TO_FULL_SECONDS / TICKS_PER_SECOND)
            .min(max_boss_hitpoints);
        self.current_boss_hitpoints().set(current_boss_hitpoints);

        let max_player_hitpoints = *bossfight_stats.hitpoints().read();
        let current_player_hitpoints = (*self.current_player_hitpoints().read()
            + max_player_hitpoints / HITPOINTS_REGENERATION_TO_FULL_SECONDS / TICKS_PER_SECOND)
            .min(max_player_hitpoints);
        self.current_player_hitpoints()
            .set(current_player_hitpoints);
    }

    fn do_rebirth<BossfightStatsLens: Copy + Readable<Target = BossfightStats>>(
        &mut self,
        bossfight_stats: Store<BossfightStats, BossfightStatsLens>,
    ) {
        self.current_boss().set(0);
        let max_boss_hitpoints = self.max_hitpoints().unwrap_or(0.0);
        let max_player_hitpoints = *bossfight_stats.hitpoints().read();

        self.current_boss_hitpoints().set(max_boss_hitpoints);
        self.current_player_hitpoints().set(max_player_hitpoints);
        self.is_fighting().set(false);
    }

    fn boss(&self) -> Option<MappedBossStore<Lens>> {
        self.bosses().get(*self.current_boss().read())
    }

    fn attack(&self) -> Option<f64> {
        self.bosses()
            .get(*self.current_boss().read())
            .map(|boss| *boss.attack().read())
    }

    fn defense(&self) -> Option<f64> {
        self.bosses()
            .get(*self.current_boss().read())
            .map(|boss| *boss.defense().read())
    }

    fn max_hitpoints(&self) -> Option<f64> {
        self.bosses()
            .get(*self.current_boss().read())
            .map(|boss| *boss.max_hitpoints().read())
    }
}

#[store(pub)]
impl<Lens> Store<BossfightStats, Lens> {
    fn do_update<
        TrainingLens: Copy + Readable<Target = Training>,
        DraftingLens: Copy + Readable<Target = Drafting>,
        RebirthLens: Copy + Readable<Target = Rebirth>,
    >(
        &mut self,
        training: Store<Training, TrainingLens>,
        drafting: Store<Drafting, DraftingLens>,
        rebirth: Store<Rebirth, RebirthLens>,
    ) {
        let attack =
            *training.attack().read() * *drafting.attack_factor().read() * *rebirth.number().read();
        let defense = *training.defense().read()
            * *drafting.defense_factor().read()
            * *rebirth.number().read();
        let hitpoints = *training.hitpoints().read()
            * *drafting.hitpoints_factor().read()
            * *rebirth.number().read();

        self.attack().set(attack);
        self.defense().set(defense);
        self.hitpoints().set(hitpoints);
    }

    fn do_rebirth<
        TrainingLens: Copy + Readable<Target = Training>,
        DraftingLens: Copy + Readable<Target = Drafting>,
        RebirthLens: Copy + Readable<Target = Rebirth>,
    >(
        &mut self,
        training: Store<Training, TrainingLens>,
        drafting: Store<Drafting, DraftingLens>,
        rebirth: Store<Rebirth, RebirthLens>,
    ) {
        self.do_update(training, drafting, rebirth);
    }
}

#[component]
pub fn BossfightView() -> Element {
    let game = use_context::<Store<Game>>();
    let bossfight_stats = game.bossfight_stats();
    let bossfight = game.bossfight();
    let boss = bossfight.boss();

    let player_hitpoints_ratio = use_memo(move || {
        bossfight.current_player_hitpoints() / *bossfight_stats.hitpoints().read()
    });

    let boss_hitpoints_ratio = use_memo(move || {
        *bossfight.current_boss_hitpoints().read()
            / bossfight.max_hitpoints().unwrap_or(f64::INFINITY)
    });

    let boss_view = if let Some(boss) = boss {
        rsx! {
            table {
                tr {
                    td {}
                    th { "Player" }
                    th { "{boss.name()}" }
                }
                tr {
                    td { "Attack" }
                    td { class: "number",
                        F64 {
                            number: bossfight_stats.attack(),
                            format_as_integer: true,
                        }
                    }
                    td { class: "number",
                        F64 { number: boss.attack(), format_as_integer: true }
                    }
                }
                tr {
                    td { "Defense" }
                    td { class: "number",
                        F64 {
                            number: bossfight_stats.defense(),
                            format_as_integer: true,
                        }
                    }
                    td { class: "number",
                        F64 { number: boss.defense(), format_as_integer: true }
                    }
                }
                tr {
                    td { "Hitpoints" }
                    td {
                        ProgressBar { progress: player_hitpoints_ratio,
                            F64 {
                                number: bossfight.current_player_hitpoints(),
                                format_as_integer: true,
                            }
                            "/"
                            F64 {
                                number: bossfight_stats.hitpoints(),
                                format_as_integer: true,
                            }
                        }
                    }
                    td {
                        ProgressBar { progress: boss_hitpoints_ratio,
                            F64 {
                                number: bossfight.current_boss_hitpoints(),
                                format_as_integer: true,
                            }
                            "/"
                            F64 {
                                number: boss.max_hitpoints(),
                                format_as_integer: true,
                            }
                        }
                    }
                }
            }
            button {
                onclick: move |_| {
                    bossfight.is_fighting().toggle();
                },
                if *bossfight.is_fighting().read() {
                    "Stop"
                } else {
                    "Fight"
                }
            }
            for paragraph in boss.unlocks().iter() {
                p { class: "boss-unlock", "({paragraph})" }
            }
            for paragraph in boss.story().iter() {
                p { {paragraph} }
            }
        }
    } else {
        rsx! {
            span { "All bosses defeated!" }
        }
    };

    rsx! {
        div { class: "vertical", style: "padding: 10px;",
            h2 { "Bossfight" }
            {boss_view}
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::game::bossfight::{Bossfight, BossfightStats};

    #[test]
    fn test_bossfight_deserialization() {
        let bossfight_stats = BossfightStats {
            attack: 2.0,
            defense: 1.0,
            hitpoints: 1e3,
        };
        let bossfight = Bossfight::new_game(&bossfight_stats);
        assert!(!bossfight.bosses.is_empty());
    }
}
