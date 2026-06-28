use dioxus::prelude::*;
use dioxus_stores::Store;
use num::{BigUint, One};

use crate::{
    game::{
        bossfight::{
            Bossfight, BossfightStats, BossfightStatsStoreExt, BossfightStatsStoreImplExt,
            BossfightStoreExt, BossfightStoreImplExt, BossfightView,
        },
        drafting::{Drafting, DraftingStoreImplExt, DraftingView},
        energy::{Energy, EnergyStoreImplExt, EnergyView},
        rebirth::{Rebirth, RebirthStoreImplExt, RebirthView},
        training::{Training, TrainingStoreExt, TrainingStoreImplExt, TrainingView},
    },
    ui::{number_format::F64, ProgressBar},
};

pub mod bossfight;
pub mod drafting;
pub mod energy;
pub mod rebirth;
pub mod training;

#[derive(Clone, Store)]
pub struct Game {
    pub energy_increment: BigUint,
    pub main_view: MainView,

    pub bossfight_stats: BossfightStats,

    pub energy: Energy,
    pub training: Training,
    pub bossfight: Bossfight,
    pub drafting: Drafting,

    pub rebirth: Rebirth,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainView {
    Training,
    Bossfight,
    Drafting,

    Rebirth,
}

impl Game {
    pub fn new_game() -> Self {
        let training = Training::new_game();
        let bossfight = Bossfight::new_game(&training);
        let bossfight_stats = BossfightStats {
            attack: training.attack,
            defense: training.defense,
            hitpoints: training.hitpoints,
        };

        Self {
            energy_increment: BigUint::one(),
            main_view: MainView::Training,

            bossfight_stats,

            energy: Energy::new_game(),
            training,
            bossfight,
            drafting: Drafting::new_game(),

            rebirth: Rebirth::new_game(),
        }
    }
}

#[store(pub)]
impl<Lens> Store<Game, Lens> {
    fn do_update(&mut self) {
        self.energy().do_update();
        self.training().do_update();
        self.drafting().do_update();
        self.rebirth().do_update();

        self.bossfight_stats()
            .do_update(self.training(), self.rebirth());
        self.bossfight().do_update(self.bossfight_stats());
    }

    fn do_rebirth(&mut self) {
        self.energy().do_rebirth();
        self.training().do_rebirth();
        self.drafting().do_rebirth();
        self.rebirth().do_rebirth(self.bossfight());

        self.bossfight_stats()
            .do_rebirth(self.training(), self.rebirth());
        self.bossfight().do_rebirth(self.bossfight_stats());
    }
}

#[component]
pub fn GameView() -> Element {
    let game = use_context::<Store<Game>>();
    let bossfight = game.bossfight();
    let bossfight_stats = game.bossfight_stats();

    let training = game.training();
    let attack = training.attack();
    let defense = training.defense();

    let player_hitpoints_ratio = use_memo(move || {
        bossfight.current_player_hitpoints() / *bossfight_stats.hitpoints().read()
    });
    let hitpoints = training.hitpoints();

    let main_view = match *game.main_view().read() {
        MainView::Training => rsx! {
            TrainingView {}
        },
        MainView::Bossfight => rsx! {
            BossfightView {}
        },
        MainView::Drafting => rsx! {
            DraftingView {}
        },
        MainView::Rebirth => rsx! {
            RebirthView {}
        },
    };

    rsx! {
        div { class: "horizontal",
            div {
                class: "vertical",
                style: "width: 400px; max-width: 400px; min-width: 400px;",
                EnergyView {}
                span {
                    "Attack: "
                    F64 { number: attack, format_as_integer: true }
                }
                span {
                    "Defense: "
                    F64 { number: defense, format_as_integer: true }
                }
                ProgressBar { progress: player_hitpoints_ratio,
                    "Hitpoints: "
                    F64 {
                        number: bossfight.current_player_hitpoints(),
                        format_as_integer: true,
                    }
                    "/"
                    F64 { number: hitpoints, format_as_integer: true }
                }
                button {
                    class: "rebirth",
                    onclick: move |_| {
                        game.main_view().set(MainView::Rebirth);
                    },
                    "Rebirth"
                }
            }
            div {
                class: "vertical",
                style: "width: 200px; max-width: 200px; min-width: 200px;",
                button {
                    onclick: move |_| {
                        game.main_view().set(MainView::Training);
                    },
                    "Training"
                }
                button {
                    onclick: move |_| {
                        game.main_view().set(MainView::Bossfight);
                    },
                    "Bossfight"
                }
                button {
                    onclick: move |_| {
                        game.main_view().set(MainView::Drafting);
                    },
                    "Drafting"
                }
            }
            {main_view}
        }
    }
}
