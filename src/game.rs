use dioxus::prelude::*;
use dioxus_stores::Store;

use crate::game::energy::{Energy, EnergyStoreImplExt, EnergyView};

pub mod energy;

#[derive(Clone, Store)]
pub struct Game {
    pub energy: Energy,
}

impl Game {
    pub fn new_game() -> Self {
        Self {
            energy: Energy::new_game(),
        }
    }
}

#[store(pub)]
impl<Lens> Store<Game, Lens> {
    fn update(&mut self) {
        self.energy().update();
    }
}

#[component]
pub fn GameView() -> Element {
    // let game = use_context::<Store<Game>>();

    rsx! {
        EnergyView {}
    }
}
