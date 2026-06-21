use dioxus::prelude::*;
use dioxus_stores::Store;
use num::{BigUint, One};

use crate::game::{
    energy::{Energy, EnergyStoreImplExt, EnergyView},
    training::{Training, TrainingStoreExt, TrainingStoreImplExt, TrainingView},
};

pub mod energy;
pub mod training;

#[derive(Clone, Store)]
pub struct Game {
    pub energy_increment: BigUint,

    pub energy: Energy,
    pub training: Training,
}

impl Game {
    pub fn new_game() -> Self {
        Self {
            energy_increment: BigUint::one(),

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
            TrainingView {}
        }
    }
}
