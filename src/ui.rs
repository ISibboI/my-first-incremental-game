use dioxus::prelude::*;

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
