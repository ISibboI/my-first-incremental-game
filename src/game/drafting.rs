use dioxus::prelude::*;

#[component]
pub fn DraftingView() -> Element {
    rsx! {
        div { class: "vertical", style: "padding: 10px;",
            h2 { "Drafting" }
        }
    }
}
