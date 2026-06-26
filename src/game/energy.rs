use dioxus::prelude::*;
use dioxus_stores::Store;
use num::BigUint;

use crate::{
    game::{Game, GameStoreExt},
    ui::ProgressBar,
    SECONDS_PER_TICK,
};

#[derive(Clone, Store)]
pub struct Energy {
    pub energy: BigUint,
    pub idle_energy: BigUint,
    pub max_energy: BigUint,

    pub energy_progress: f64,
    pub energy_progress_per_tick: f64,

    pub max_energy_after_rebirth: BigUint,
    pub max_energy_after_rebirth_progress: f64,
    pub max_energy_after_rebirth_progress_increment: f64,
}

impl Energy {
    pub fn new_game() -> Self {
        let initial_max_energy = 100u8;

        Self {
            energy: BigUint::from(0u8),
            idle_energy: BigUint::from(0u8),
            max_energy: BigUint::from(initial_max_energy),

            energy_progress: 0.0,
            energy_progress_per_tick: SECONDS_PER_TICK,

            max_energy_after_rebirth: BigUint::from(initial_max_energy),
            max_energy_after_rebirth_progress: 0.0,
            max_energy_after_rebirth_progress_increment: 1.0 / 60.0,
        }
    }
}

#[store(pub)]
impl<Lens> Store<Energy, Lens> {
    fn do_update(&mut self) {
        // Increase energy progress.
        let energy_progress =
            *self.energy_progress().read() + *self.energy_progress_per_tick().read();
        self.energy_progress().set(energy_progress);

        // If energy progress is full, increase energy and reset energy progress.
        if *self.energy_progress().read() >= 1.0 - 1e-6 {
            if *self.energy().read() < *self.max_energy().read() {
                self.energy().with_mut(|energy| *energy += 1u8);
                self.idle_energy()
                    .with_mut(|idle_energy| *idle_energy += 1u8);
            }
            self.energy_progress().set(0.0);

            // Increase max_energy_after_rebirth progress.
            let max_energy_after_rebirth_progress =
                *self.max_energy_after_rebirth_progress().read()
                    + *self.max_energy_after_rebirth_progress_increment().read();
            self.max_energy_after_rebirth_progress()
                .set(max_energy_after_rebirth_progress);

            // If max_energy_after_rebirth progress is full, increase max_energy_after_rebirth and reset max_energy_after_rebirth progress.
            if *self.max_energy_after_rebirth_progress().read() >= 1.0 - 1e-6 {
                self.max_energy_after_rebirth_progress().set(0.0);
                self.max_energy_after_rebirth()
                    .with_mut(|max_energy_after_rebirth| *max_energy_after_rebirth += 1u8);
            }
        }
    }

    fn do_rebirth(&mut self) {
        self.energy().set(0u8.into());
        self.idle_energy().set(0u8.into());
        let max_energy_after_rebirth = self.max_energy_after_rebirth().read().clone();
        self.max_energy().set(max_energy_after_rebirth);

        self.energy_progress().set(0.0);

        self.max_energy_after_rebirth_progress().set(0.0);
    }
}

#[component]
pub fn EnergyView() -> Element {
    let game = use_context::<Store<Game>>();
    let energy = game.energy();

    let current_energy = energy.energy();
    let idle_energy = energy.idle_energy();
    let max_energy = energy.max_energy();
    let energy_progress = energy.energy_progress();
    let max_energy_after_rebirth = energy.max_energy_after_rebirth();
    let max_energy_after_rebirth_progress = energy.max_energy_after_rebirth_progress();

    // let energy_progress_text = use_memo(move || {});

    rsx! {
        div { class: "horizontal",
            div { class: "vertical",
                ProgressBar {
                    progress: energy_progress,
                    text: "Energy: {current_energy}/{max_energy}; Idle: {idle_energy}",
                }
                ProgressBar {
                    progress: max_energy_after_rebirth_progress,
                    text: "Max energy after rebirth: {max_energy_after_rebirth}",
                }
            }
        }
    }
}
