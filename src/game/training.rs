use dioxus::prelude::*;
use dioxus_stores::Store;
use num::ToPrimitive;

use crate::{
    game::{energy::EnergyStoreExt, Game, GameStoreExt},
    ui::{
        number_format::{U32, U64},
        BigUintToU32ShiftButton, EnergyIncrementSelector, ProgressBar, U32ToBigUintShiftButton,
    },
};

pub const BASE_ATTACK: f64 = 1.0;
pub const BASE_DEFENSE: f64 = 1.0;
pub const BASE_HITPOINTS: f64 = 1000.0;

#[derive(Clone, Store)]
pub struct Training {
    pub attack: f64,
    pub defense: f64,
    pub hitpoints: f64,

    pub attack_skills: Vec<Skill>,
    pub defense_skills: Vec<Skill>,
    pub hitpoint_skills: Vec<Skill>,
}

#[derive(Clone, Store)]
pub struct Skill {
    pub name: String,
    pub assigned_energy: u32,
    pub progress: u32,
    pub required_progress: u32,
    pub required_progress_after_rebirth: f64,
    pub required_progress_after_rebirth_factor: f64,
    pub is_required_progress_after_rebirth_minimum_reached: bool,
    pub level: u64,
    pub value_per_level: f64,
    pub unlocked: bool,
}

impl Training {
    pub fn new_game() -> Self {
        Self {
            attack: BASE_ATTACK,
            defense: BASE_DEFENSE,
            hitpoints: BASE_HITPOINTS,

            attack_skills: vec![
                Skill::new_game("Strength", 2000, 1.0, true),
                Skill::new_game("Punch", 10000, 10.0, false),
                Skill::new_game("Sword", 30000, 100.0, false),
                Skill::new_game("Basic combo", 60000, 1000.0, false),
                Skill::new_game("Awesome combo", 100000, 10000.0, false),
            ],
            defense_skills: vec![
                Skill::new_game("Flexibility", 2000, 1.0, true),
                Skill::new_game("Evasion", 10000, 10.0, false),
                Skill::new_game("Limbo", 30000, 100.0, false),
                Skill::new_game("Parry", 60000, 1000.0, false),
                Skill::new_game("Catch arrows", 100000, 10000.0, false),
            ],
            hitpoint_skills: vec![
                Skill::new_game("Healthy nutrition", 2000, 1e3, true),
                Skill::new_game("Thicker skin", 10000, 1e4, false),
                Skill::new_game("Pain endurance", 30000, 1e5, false),
                Skill::new_game("Sleep on bed of nails", 60000, 1e6, false),
                Skill::new_game("Simply don't die", 100000, 1e7, false),
            ],
        }
    }
}

impl Skill {
    pub fn new_game(
        name: impl ToString,
        required_progress: u32,
        value_per_level: f64,
        unlocked: bool,
    ) -> Self {
        Self {
            name: name.to_string(),
            assigned_energy: 0,
            progress: 0,
            required_progress,
            required_progress_after_rebirth: required_progress as f64,
            required_progress_after_rebirth_factor: 1.0 - 1e-4,
            is_required_progress_after_rebirth_minimum_reached: required_progress == 1,
            level: 0,
            value_per_level,
            unlocked,
        }
    }
}

#[store(pub)]
impl<Lens> Store<Training, Lens> {
    fn do_update(&mut self) {
        // Update skills.
        for mut skill in self
            .attack_skills()
            .iter()
            .chain(self.defense_skills().iter())
            .chain(self.hitpoint_skills().iter())
        {
            skill.do_update();
        }

        // Unlock skills.
        const UNLOCK_REQUIREMENT: u64 = 5000;
        let mut previous_level = 0;
        for (index, skill) in self.attack_skills().iter().enumerate() {
            if !*skill.unlocked().read() {
                let required_level = UNLOCK_REQUIREMENT * index as u64;
                if previous_level >= required_level {
                    skill.unlocked().set(true);
                    break;
                }
            }
            previous_level = *skill.level().read();
        }
        let mut previous_level = 0;
        for (index, skill) in self.defense_skills().iter().enumerate() {
            if !*skill.unlocked().read() {
                let required_level = UNLOCK_REQUIREMENT * index as u64;
                if previous_level >= required_level {
                    skill.unlocked().set(true);
                    break;
                }
            }
            previous_level = *skill.level().read();
        }
        let mut previous_level = 0;
        for (index, skill) in self.hitpoint_skills().iter().enumerate() {
            if !*skill.unlocked().read() {
                let required_level = UNLOCK_REQUIREMENT * index as u64;
                if previous_level >= required_level {
                    skill.unlocked().set(true);
                    break;
                }
            }
            previous_level = *skill.level().read();
        }

        // Update stats.
        self.attack().set(
            BASE_ATTACK
                + self
                    .attack_skills()
                    .iter()
                    .map(|skill| skill.value())
                    .sum::<f64>(),
        );
        self.defense().set(
            BASE_DEFENSE
                + self
                    .defense_skills()
                    .iter()
                    .map(|skill| skill.value())
                    .sum::<f64>(),
        );
        self.hitpoints().set(
            BASE_HITPOINTS
                + self
                    .hitpoint_skills()
                    .iter()
                    .map(|skill| skill.value())
                    .sum::<f64>(),
        );
    }

    fn do_rebirth(&mut self) {
        self.attack().set(BASE_ATTACK);
        self.defense().set(BASE_DEFENSE);
        self.hitpoints().set(BASE_HITPOINTS);

        for (index, mut skill) in self
            .attack_skills()
            .iter()
            .enumerate()
            .chain(self.defense_skills().iter().enumerate())
            .chain(self.hitpoint_skills().iter().enumerate())
        {
            skill.do_rebirth();
            skill.unlocked().set(index == 0);
        }
    }
}

#[store(pub)]
impl<Lens> Store<Skill, Lens> {
    fn do_update(&mut self) {
        // Increase progress.
        let progress = *self.assigned_energy().read() + *self.progress().read();
        self.progress().set(progress);

        let required_progress = *self.required_progress().read();
        if progress >= required_progress {
            // Increase level.
            self.progress().set(0);
            self.level().with_mut(|level| *level += 1);

            // Decrease required progress after rebirth.
            let factor = *self.required_progress_after_rebirth_factor().read();
            let is_required_progress_after_rebirth_minimum_reached = self
                .required_progress_after_rebirth()
                .with_mut(|required_progress_after_rebirth| {
                    *required_progress_after_rebirth *= factor;
                    let minimum_required_progress_after_rebirth =
                        1.max((required_progress as f64 * 0.89).floor() as u32);
                    if *required_progress_after_rebirth
                        < minimum_required_progress_after_rebirth as f64
                    {
                        *required_progress_after_rebirth =
                            minimum_required_progress_after_rebirth as f64;
                        true
                    } else {
                        false
                    }
                });
            self.is_required_progress_after_rebirth_minimum_reached()
                .set(is_required_progress_after_rebirth_minimum_reached);
        }
    }

    fn do_rebirth(&mut self) {
        self.assigned_energy().set(0);
        self.progress().set(0);
        self.level().set(0);

        let required_progress = self.required_progress_after_rebirth_u32();
        self.required_progress().set(required_progress);
        self.required_progress_after_rebirth()
            .set(required_progress as f64);
        self.is_required_progress_after_rebirth_minimum_reached()
            .set(required_progress == 1);
    }

    fn value(&self) -> f64 {
        *self.level().read() as f64 * *self.value_per_level().read()
    }

    fn required_progress_after_rebirth_u32(&self) -> u32 {
        self.required_progress_after_rebirth().read().ceil() as u32
    }
}

#[component]
pub fn TrainingView() -> Element {
    let game = use_context::<Store<Game>>();
    let training = game.training();

    rsx! {
        div { class: "vertical", style: "padding: 10px;",
            h2 { "Training" }
            EnergyIncrementSelector {}
            div { class: "vertical",
                table {
                    tr {
                        th { "Attack skill" }
                        th { "Level" }
                        th { "Assigned Energy" }
                        th { "Cap" }
                        th { "Cap after rebirth" }
                    }
                    for skill in training.attack_skills().iter() {
                        SkillView { skill }
                    }
                    tr {
                        th { "Defense skill" }
                        th { "Level" }
                        th { "Assigned Energy" }
                        th { "Cap" }
                        th { "Cap after rebirth" }
                    }
                    for skill in training.defense_skills().iter() {
                        SkillView { skill }
                    }
                    tr {
                        th { "Survival skill" }
                        th { "Level" }
                        th { "Assigned Energy" }
                        th { "Cap" }
                        th { "Cap after rebirth" }
                    }
                    for skill in training.hitpoint_skills().iter() {
                        SkillView { skill }
                    }
                }
            }
        }
    }
}

#[component]
pub fn SkillView(skill: WriteStore<Skill>) -> Element {
    let game = use_context::<Store<Game>>();
    let energy = game.energy();

    let energy_increment = game.energy_increment();
    let energy_increment_u32 =
        use_memo(move || energy_increment.read().to_u32().unwrap_or(u32::MAX));

    let progress = use_memo(move || {
        let progress = *skill.progress().read();
        let required_progress = *skill.required_progress().read();
        progress as f64 / required_progress as f64
    });

    rsx! {
        if *skill.unlocked().read() {
            tr {
                td {
                    ProgressBar { progress, {skill.name()} }
                }
                td { class: "number",
                    U64 { number: skill.level() }
                }
                td { class: "number",
                    U32 { number: skill.assigned_energy() }
                }
                td { class: "number",
                    U32 { number: skill.required_progress() }
                }
                td { class: if *skill.is_required_progress_after_rebirth_minimum_reached().read() { "number-capped" } else { "number" },
                    U32 { number: skill.required_progress_after_rebirth_u32() }
                }
                td {
                    BigUintToU32ShiftButton {
                        from: energy.idle_energy(),
                        to: skill.assigned_energy(),
                        amount: energy_increment,
                        text: "+",
                    }
                }
                td {
                    U32ToBigUintShiftButton {
                        from: skill.assigned_energy(),
                        to: energy.idle_energy(),
                        amount: energy_increment_u32,
                        text: "-",
                    }
                }
            }
        } else {
            tr {
                td {
                    ProgressBar { progress: 0.0, "<Locked>" }
                }
            }
        }
    }
}
