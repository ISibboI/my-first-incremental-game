use std::ops::{AddAssign, SubAssign};

use dioxus::prelude::*;
use num::{BigUint, Integer, ToPrimitive};

#[component]
pub fn ProgressBar(progress: ReadSignal<f64>, text: ReadSignal<String>) -> Element {
    rsx! {
        div { class: "progress-bar",
            div {
                class: "progress-bar-fill",
                style: format!("width: {}%;", *progress.read() * 100.0),
            }
            span { class: "progress-bar-text", "{text}" }
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
            onclick: move |event| {
                event.prevent_default();

                let actual_amount = amount.read().clone().min(from.read().clone());
                from.with_mut(|from| *from -= &actual_amount);
                to.with_mut(|to| *to = *to + actual_amount.to_u32().unwrap());
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
            onclick: move |event| {
                event.prevent_default();

                let actual_amount = amount.read().min(*from.read());
                from.with_mut(|from| *from -= actual_amount);
                to.with_mut(|to| *to += actual_amount);
            },
            "{text}"
        }
    }
}
