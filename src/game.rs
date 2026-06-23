use dioxus::prelude::*;
use dioxus_stores::Store;
use num::{BigUint, One};

use crate::game::{
    drafting::DraftingView,
    energy::{Energy, EnergyStoreImplExt, EnergyView},
    training::{Training, TrainingStoreExt, TrainingStoreImplExt, TrainingView},
};

pub mod drafting;
pub mod energy;
pub mod training;

#[derive(Clone, Store)]
pub struct Game {
    pub energy_increment: BigUint,
    pub main_view: MainView,

    pub energy: Energy,
    pub training: Training,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainView {
    Training,
    Drafting,
}

impl Game {
    pub fn new_game() -> Self {
        Self {
            energy_increment: BigUint::one(),
            main_view: MainView::Training,

            energy: Energy::new_game(),
            training: Training::new_game(),
        }
    }
}

#[store(pub)]
impl<Lens> Store<Game, Lens> {
    fn update(&mut self) {
        self.energy().update();
        self.training().update();
    }

    fn rebirth(&mut self) {
        self.energy().rebirth();
        self.training().rebirth();
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
        MainView::Drafting => rsx! {
            DraftingView {}
        },
    };

    rsx! {
        div { class: "horizontal",
            div { class: "vertical",
                EnergyView {}
                span { "Attack: {attack}" }
                span { "Defense: {defense}" }
                span { "Hitpoints: {hitpoints}" }
                button {
                    class: "rebirth",
                    onclick: move |event| {
                        event.prevent_default();
                        game.rebirth();
                    },
                    "Rebirth"
                }
            }
            div { class: "vertical",
                button {
                    onclick: move |event| {
                        event.prevent_default();
                        game.main_view().set(MainView::Training);
                    },
                    "Training"
                }
                button {
                    onclick: move |event| {
                        event.prevent_default();
                        game.main_view().set(MainView::Drafting);
                    },
                    "Drafting"
                }
            }
            {main_view}
        }
    }
}
