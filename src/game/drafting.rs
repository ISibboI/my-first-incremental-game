use dioxus::prelude::*;

#[derive(Clone, Store)]
pub struct Drafting {}

impl Drafting {
    pub fn new_game() -> Self {
        Self {}
    }
}

#[store(pub)]
impl<Lens> Store<Drafting, Lens> {
    fn update(&mut self) {}

    fn rebirth(&mut self) {}
}

#[component]
pub fn DraftingView() -> Element {
    rsx! {
        div { class: "vertical", style: "padding: 10px;",
            h2 { "Drafting" }
        }
    }
}
