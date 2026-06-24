use dioxus::prelude::*;
use dioxus_stores::index::IndexWrite;

use crate::{
    game::{
        training::{Training, TrainingStoreExt},
        Game, GameStoreExt,
    },
    ui::ProgressBar,
    TICKS_PER_SECOND,
};

const HITPOINTS_REGENERATION_TO_FULL_SECONDS: f64 = 60.0;

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
    pub attack: f64,
    pub defense: f64,
    pub max_hitpoints: f64,
}

impl Bossfight {
    pub fn new_game(training: &Training) -> Self {
        let bosses: Vec<_> = ["A flea", "A rat", "A wolf", "A bear", "A dragon"]
            .into_iter()
            .enumerate()
            .map(|(level, name)| {
                let factor = 10.0f64.powi(level.try_into().unwrap());
                Boss::new(name, 2.0 * factor, 1.0 * factor, 1e3 * factor)
            })
            .collect();
        let current_boss_hitpoints = bosses.first().map_or(0.0, |boss| boss.max_hitpoints);

        Self {
            bosses,

            current_boss: 0,
            current_boss_hitpoints,
            current_player_hitpoints: training.hitpoints,
            is_fighting: false,
        }
    }
}

impl Boss {
    pub fn new(name: impl ToString, attack: f64, defense: f64, max_hitpoints: f64) -> Self {
        Self {
            name: name.to_string(),
            attack,
            defense,
            max_hitpoints,
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
    fn update<TrainingLens: Copy + Readable<Target = Training>>(
        &mut self,
        training: Store<Training, TrainingLens>,
    ) {
        if *self.is_fighting().read() {
            let damage_dealt = (*training.attack().read()
                - self.boss().map_or(0.0, |boss| *boss.defense().read()))
            .max(0.0);
            let damage_taken = (self.boss().map_or(0.0, |boss| *boss.attack().read())
                - *training.defense().read())
            .max(0.0);

            self.current_boss_hitpoints().with_mut(|hp| {
                *hp -= damage_dealt;
                *hp = hp.max(0.0);
            });
            self.current_player_hitpoints().with_mut(|hp| {
                *hp -= damage_taken;
                *hp = hp.max(0.0);
            });

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

        let max_boss_hitpoints = self.boss().map_or(0.0, |boss| *boss.max_hitpoints().read());
        let current_boss_hitpoints = (*self.current_boss_hitpoints().read()
            + max_boss_hitpoints / HITPOINTS_REGENERATION_TO_FULL_SECONDS / TICKS_PER_SECOND)
            .min(max_boss_hitpoints);
        self.current_boss_hitpoints().set(current_boss_hitpoints);

        let max_player_hitpoints = *training.hitpoints().read();
        let current_player_hitpoints = (*self.current_player_hitpoints().read()
            + max_player_hitpoints / HITPOINTS_REGENERATION_TO_FULL_SECONDS / TICKS_PER_SECOND)
            .min(max_player_hitpoints);
        self.current_player_hitpoints()
            .set(current_player_hitpoints);
    }

    fn rebirth<TrainingLens: Copy + Readable<Target = Training>>(
        &mut self,
        training: Store<Training, TrainingLens>,
    ) {
        let max_boss_hitpoints = self.max_hitpoints().unwrap_or(0.0);
        let max_player_hitpoints = *training.hitpoints().read();

        self.current_boss().set(0);
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

#[component]
pub fn BossfightView() -> Element {
    let game = use_context::<Store<Game>>();
    let training = game.training();
    let bossfight = game.bossfight();
    let boss = bossfight.boss();

    let player_hitpoints_ratio =
        use_memo(move || bossfight.current_player_hitpoints() / *training.hitpoints().read());

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
                    td { "{training.attack()}" }
                    td { "{boss.attack()}" }
                }
                tr {
                    td { "Defense" }
                    td { "{training.defense()}" }
                    td { "{boss.defense()}" }
                }
                tr {
                    td { "Hitpoints" }
                    td {
                        ProgressBar {
                            progress: player_hitpoints_ratio,
                            text: "{bossfight.current_player_hitpoints()}/{training.hitpoints()}",
                        }
                    }
                    td {
                        ProgressBar {
                            progress: boss_hitpoints_ratio,
                            text: "{bossfight.current_boss_hitpoints()}/{boss.max_hitpoints()}",
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
