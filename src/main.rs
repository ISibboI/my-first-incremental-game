use std::time::Duration;

use dioxus::prelude::*;
use dioxus_sdk::time::use_interval;

use crate::game::{Game, GameStoreImplExt, GameView};

const FAVICON: Asset = asset!("/assets/favicon.ico");
const MAIN_CSS: Asset = asset!("/assets/main.css");
const TAILWIND_CSS: Asset = asset!("/assets/tailwind.css");

const TICK_RATE: Duration = Duration::from_millis(20);
#[expect(dead_code)]
const TICKS_PER_SECOND: f64 = 1.0 / TICK_RATE.as_secs_f64();
const SECONDS_PER_TICK: f64 = TICK_RATE.as_secs_f64();

mod game;
mod ui;

fn main() {
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    let mut game = use_store(Game::new_game);
    use_context_provider(move || game);
    use_interval(TICK_RATE, move |()| game.update());

    rsx! {
        document::Link { rel: "icon", href: FAVICON }
        document::Link { rel: "stylesheet", href: MAIN_CSS }
        document::Link { rel: "stylesheet", href: TAILWIND_CSS }

        GameView {}
    }
}
