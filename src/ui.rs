use dioxus::prelude::*;
use num::{BigUint, ToPrimitive};

use crate::game::{energy::EnergyStoreExt, Game, GameStoreExt};

pub mod number_format;

#[component]
pub fn ProgressBar(progress: ReadSignal<f64>, children: Element) -> Element {
    rsx! {
        div { class: "progress-bar",
            div {
                class: "progress-bar-fill",
                style: format!("width: {}%;", *progress.read() * 100.0),
            }
            span { class: "progress-bar-text", {children} }
        }
    }
}

#[component]
pub fn BigUintInput(number: WriteSignal<BigUint>) -> Element {
    rsx! {
        input {
            r#type: "text",
            value: "{number}",
            oninput: move |event| {
                if let Ok(parsed) = event.value().parse::<BigUint>() {
                    number.set(parsed);
                }
            },
        }
    }
}

/// A button that shifts a value from one variable to another by a certain amount.
/// "Shifting" means decreasing the value on one end and increasing it on the other end.
#[component]
pub fn BigUintToU32ShiftButton(
    from: WriteSignal<BigUint>,
    to: WriteSignal<u32>,
    amount: ReadSignal<BigUint>,
    text: ReadSignal<String>,
) -> Element {
    rsx! {
        button {
            onclick: move |_| {
                let actual_amount = amount.read().clone().min(from.read().clone());
                from.with_mut(|from| *from -= &actual_amount);
                to.with_mut(|to| *to += actual_amount.to_u32().unwrap());
            },
            "{text}"
        }
    }
}

/// A button that shifts a value from one variable to another by a certain amount.
/// "Shifting" means decreasing the value on one end and increasing it on the other end.
#[component]
pub fn U32ToBigUintShiftButton(
    from: WriteSignal<u32>,
    to: WriteSignal<BigUint>,
    amount: ReadSignal<u32>,
    text: ReadSignal<String>,
) -> Element {
    rsx! {
        button {
            onclick: move |_| {
                let actual_amount = amount.read().min(*from.read());
                from.with_mut(|from| *from -= actual_amount);
                to.with_mut(|to| *to += actual_amount);
            },
            "{text}"
        }
    }
}

#[component]
pub fn EnergyIncrementSelector() -> Element {
    let game = use_context::<Store<Game>>();
    let energy = game.energy();
    let mut energy_increment = game.energy_increment();

    let cap = use_memo(move || energy.max_energy().read().clone());
    let idle = use_memo(move || energy.idle_energy().read().clone());

    rsx! {
        div { class: "horizontal",
            BigUintInput { number: energy_increment }
            button {
                onclick: move |_| {
                    energy_increment.set(cap());
                },
                "Cap"
            }
            button {
                onclick: move |_| {
                    energy_increment.set(cap() / 3u32);
                },
                "Cap/3"
            }
            button {
                onclick: move |_| {
                    energy_increment.set(idle());
                },
                "Idle"
            }
            button {
                onclick: move |_| {
                    energy_increment.set(idle() / 3u32);
                },
                "Idle/3"
            }
        }
    }
}
