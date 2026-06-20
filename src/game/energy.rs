use dioxus::prelude::*;
use dioxus_stores::Store;
use num::BigUint;

use crate::{
    game::{Game, GameStoreExt},
    SECONDS_PER_TICK,
};

#[derive(Clone, Store)]
pub struct Energy {
    pub energy: BigUint,
    pub max_energy: BigUint,
    pub energy_progress: f64,
    pub energy_progress_per_tick: f64,
}

impl Energy {
    pub fn new_game() -> Self {
        Self {
            energy: BigUint::from(0u32),
            max_energy: BigUint::from(10u32),
            energy_progress: 0.0,
            energy_progress_per_tick: SECONDS_PER_TICK,
        }
    }
}

#[store(pub)]
impl<Lens> Store<Energy, Lens> {
    fn update(&mut self) {
        let energy_progress =
            *self.energy_progress().read() + *self.energy_progress_per_tick().read();
        *self.energy_progress().write() = energy_progress;

        if *self.energy_progress().read() >= 1.0 - 1e-6 {
            if *self.energy().read() < *self.max_energy().read() {
                *self.energy().write() += 1u8;
            }
            *self.energy_progress().write() = 0.0;
        }
    }
}

#[component]
pub fn EnergyView() -> Element {
    let game = use_context::<Store<Game>>();
    let energy = game.energy();

    let current_energy = energy.energy();
    let max_energy = energy.max_energy();
    let energy_progress = energy.energy_progress();
    let energy_progress_percent = use_memo(move || format!("{:.0}%", energy_progress * 100.0));

    rsx! {
        div {
            span { "Energy: {current_energy}/{max_energy}" }
            progress { value: energy_progress, max: 1.0, "Energy Progress: {energy_progress_percent}" }
        }
    }
}
