use dioxus::prelude::*;
use dioxus_stores::Store;
use num::{BigUint, One, ToPrimitive};

use crate::{
    game::{energy::EnergyStoreExt, Game, GameStoreExt},
    ui::{BigUintInput, BigUintToU32ShiftButton, ProgressBar, U32ToBigUintShiftButton},
};

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
    pub level: u64,
    pub value_per_level: f64,
    pub unlocked: bool,
}

impl Training {
    pub fn new_game() -> Self {
        Self {
            attack: 1.0,
            defense: 1.0,
            hitpoints: 10.0,

            attack_skills: vec![
                Skill::new_game("Push ups", 1000, 1.0, true),
                Skill::new_game("Punch", 3000, 10.0, false),
                Skill::new_game("Sword", 10000, 100.0, false),
                Skill::new_game("Basic combo", 30000, 1000.0, false),
                Skill::new_game("Awesome combo", 100000, 10000.0, false),
            ],
            defense_skills: vec![
                Skill::new_game("Flexibility", 1000, 1.0, true),
                Skill::new_game("Evasion", 3000, 10.0, false),
                Skill::new_game("Limbo", 10000, 100.0, false),
                Skill::new_game("Parry", 30000, 1000.0, false),
                Skill::new_game("Catch arrows", 100000, 10000.0, false),
            ],
            hitpoint_skills: vec![
                Skill::new_game("Healthy nutrition", 1000, 10.0, true),
                Skill::new_game("Thicker skin", 3000, 100.0, false),
                Skill::new_game("Pain endurance", 10000, 1000.0, false),
                Skill::new_game("Sleep on bed of nails", 30000, 10000.0, false),
                Skill::new_game("Simply don't die", 100000, 100000.0, false),
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
            level: 0,
            value_per_level,
            unlocked,
        }
    }
}

#[store(pub)]
impl<Lens> Store<Training, Lens> {
    fn update(&mut self) {
        for mut skill in self
            .attack_skills()
            .iter()
            .chain(self.defense_skills().iter())
            .chain(self.hitpoint_skills().iter())
        {
            skill.update();
        }

        self.attack().set(
            1.0 + self
                .attack_skills()
                .iter()
                .map(|skill| skill.value())
                .sum::<f64>(),
        );
        self.defense().set(
            1.0 + self
                .defense_skills()
                .iter()
                .map(|skill| skill.value())
                .sum::<f64>(),
        );
        self.hitpoints().set(
            10.0 + self
                .hitpoint_skills()
                .iter()
                .map(|skill| skill.value())
                .sum::<f64>(),
        );
    }

    fn rebirth(&mut self) {
        self.attack().set(1.0);
        self.defense().set(1.0);
        self.hitpoints().set(10.0);

        for mut skill in self
            .attack_skills()
            .iter()
            .chain(self.defense_skills().iter())
            .chain(self.hitpoint_skills().iter())
        {
            skill.rebirth();
        }
    }
}

#[store(pub)]
impl<Lens> Store<Skill, Lens> {
    fn update(&mut self) {
        let progress = *self.assigned_energy().read() + *self.progress().read();
        self.progress().set(progress);

        if *self.progress().read() >= *self.required_progress().read() {
            self.progress().set(0);
            self.level().with_mut(|level| *level += 1);
        }
    }

    fn rebirth(&mut self) {
        self.assigned_energy().set(0);
        self.progress().set(0);
        self.level().set(0);
    }

    fn value(&self) -> f64 {
        *self.level().read() as f64 * *self.value_per_level().read()
    }
}

#[component]
pub fn TrainingView() -> Element {
    let game = use_context::<Store<Game>>();
    let training = game.training();

    let energy_increment = game.energy_increment();

    let attack = training.attack();
    let defense = training.defense();
    let hitpoints = training.hitpoints();

    rsx! {
        div { class: "vertical", style: "padding: 10px;",
            h2 { "Training" }
            div { class: "horizontal",
                BigUintInput { number: energy_increment }
            }
            div { class: "vertical",
                table {
                    tr {
                        th { "Attack skill" }
                        th { "Level" }
                        th { "Assigned Energy" }
                    }
                    for skill in training.attack_skills().iter() {
                        SkillView { skill }
                    }
                    tr {
                        th { "Defense skill" }
                        th { "Level" }
                        th { "Assigned Energy" }
                    }
                    for skill in training.defense_skills().iter() {
                        SkillView { skill }
                    }
                    tr {
                        th { "Survival skill" }
                        th { "Level" }
                        th { "Assigned Energy" }
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
                    ProgressBar { progress, text: skill.name() }
                }
                td { class: "number", "{skill.level()}" }
                td { class: "number", "{skill.assigned_energy()}" }
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
                    ProgressBar { progress: 0.0, text: "<Locked>" }
                }
            }
        }
    }
}
