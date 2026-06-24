use dioxus::prelude::*;
use dioxus_stores::Store;
use num::{BigUint, One};

use crate::game::{
    bossfight::{Bossfight, BossfightStoreImplExt, BossfightView},
    drafting::{Drafting, DraftingStoreImplExt, DraftingView},
    energy::{Energy, EnergyStoreImplExt, EnergyView},
    training::{Training, TrainingStoreExt, TrainingStoreImplExt, TrainingView},
};

pub mod bossfight;
pub mod drafting;
pub mod energy;
pub mod training;

#[derive(Clone, Store)]
pub struct Game {
    pub energy_increment: BigUint,
    pub main_view: MainView,

    pub energy: Energy,
    pub training: Training,
    pub bossfight: Bossfight,
    pub drafting: Drafting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainView {
    Training,
    Bossfight,
    Drafting,
}

impl Game {
    pub fn new_game() -> Self {
        let training = Training::new_game();
        let bossfight = Bossfight::new_game(&training);

        Self {
            energy_increment: BigUint::one(),
            main_view: MainView::Training,

            energy: Energy::new_game(),
            training,
            bossfight,
            drafting: Drafting::new_game(),
        }
    }
}

#[store(pub)]
impl<Lens> Store<Game, Lens> {
    fn update(&mut self) {
        self.energy().update();
        self.training().update();
        self.bossfight().update(self.training());
        self.drafting().update();
    }

    fn rebirth(&mut self) {
        self.energy().rebirth();
        self.training().rebirth();
        self.bossfight().rebirth(self.training());
        self.drafting().rebirth();
    }
}

#[component]
pub fn GameView() -> Element {
    let mut game = use_context::<Store<Game>>();

    let training = game.training();
    let attack = training.attack();
    let defense = training.defense();
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
    };

    rsx! {
        div { class: "horizontal",
            div {
                class: "vertical",
                style: "width: 400px; max-width: 400px; min-width: 400px;",
                EnergyView {}
                span { "Attack: {attack}" }
                span { "Defense: {defense}" }
                span { "Hitpoints: {hitpoints}" }
                button {
                    class: "rebirth",
                    onclick: move |_| {
                        game.rebirth();
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
