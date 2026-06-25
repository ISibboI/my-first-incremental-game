use dioxus::prelude::*;

use crate::game::{Game, GameStoreExt, GameStoreImplExt, MainView};

#[derive(Clone, Store)]
pub struct Rebirth {}

impl Rebirth {
    pub fn new_game() -> Self {
        Self {}
    }
}

#[store(pub)]
impl<Lens> Store<Rebirth, Lens> {
    fn do_update(&mut self) {}

    fn do_rebirth(&mut self) {}
}

#[component]
pub fn RebirthView() -> Element {
    let mut game = use_context::<Store<Game>>();

    rsx! {
        div { class: "vertical", style: "padding: 10px;",
            h2 { "Rebirth" }
            button {
                class: "rebirth",
                onclick: move |_| {
                    game.do_rebirth();
                    game.main_view().set(MainView::Training);
                },
                "Rebirth"
            }
        }
    }
}
