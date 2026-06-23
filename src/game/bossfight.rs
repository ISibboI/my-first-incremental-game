use dioxus::prelude::*;
use dioxus_stores::index::IndexWrite;

use crate::game::{Game, GameStoreExt};

#[derive(Clone, Store)]
pub struct Bossfight {
    pub bosses: Vec<Boss>,

    pub current_boss: usize,
    pub current_boss_health: f64,
}

#[derive(Clone, Store)]
pub struct Boss {
    pub name: String,
    pub attack: f64,
    pub defense: f64,
    pub max_health: f64,
}

impl Bossfight {
    pub fn new_game() -> Self {
        Self {
            bosses: ["A flea", "A rat", "A wolf", "A bear", "A dragon"]
                .into_iter()
                .enumerate()
                .map(|(level, name)| {
                    let factor = 10.0f64.powi(level.try_into().unwrap());
                    Boss::new(name, 2.0 * factor, 1.0 * factor, 1e3 * factor)
                })
                .collect(),

            current_boss: 0,
            current_boss_health: 0.0,
        }
    }
}

impl Boss {
    pub fn new(name: impl ToString, attack: f64, defense: f64, max_health: f64) -> Self {
        Self {
            name: name.to_string(),
            attack,
            defense,
            max_health,
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
    fn update(&mut self) {}

    fn rebirth(&mut self) {}

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

    fn max_health(&self) -> Option<f64> {
        self.bosses()
            .get(*self.current_boss().read())
            .map(|boss| *boss.max_health().read())
    }
}

#[component]
pub fn BossfightView() -> Element {
    let game = use_context::<Store<Game>>();
    let bossfight = game.bossfight();
    let boss = bossfight.boss();

    let boss_view = if let Some(boss) = boss {
        rsx! {
            table {
                tr {
                    td { "Boss" }
                    td { "{boss.name()}" }
                }
                tr {
                    td { "Attack" }
                    td { "{boss.attack()}" }
                }
                tr {
                    td { "Defense" }
                    td { "{boss.defense()}" }
                }
                tr {
                    td { "Health" }
                    td { "{bossfight.current_boss_health()}/{boss.max_health()}" }
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
