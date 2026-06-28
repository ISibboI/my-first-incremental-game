use dioxus::prelude::*;
use jiff::Zoned;

use crate::{
    game::{
        bossfight::{Bossfight, BossfightStoreExt},
        Game, GameStoreExt, GameStoreImplExt, MainView,
    },
    ui::number_format::F64,
};

#[derive(Clone, Store)]
pub struct Rebirth {
    pub last_rebirth_timestamp: Zoned,
    pub current_timestamp: Zoned,
    pub number: f64,
}

impl Rebirth {
    pub fn new_game() -> Self {
        let current_timestamp = Zoned::now();
        Self {
            last_rebirth_timestamp: current_timestamp.clone(),
            current_timestamp,
            number: 1.0,
        }
    }
}

#[store(pub)]
impl<Lens> Store<Rebirth, Lens> {
    fn do_update(&mut self) {
        self.current_timestamp().set(Zoned::now());
    }

    fn do_rebirth<BossfightLens: Copy + Readable<Target = Bossfight>>(
        &mut self,
        bossfight: Store<Bossfight, BossfightLens>,
    ) {
        let number = self.rebirth_number(bossfight);
        self.number().set(number);

        self.last_rebirth_timestamp().set(Zoned::now());
    }

    fn boss_factor<BossfightLens: Copy + Readable<Target = Bossfight>>(
        &self,
        bossfight: Store<Bossfight, BossfightLens>,
    ) -> f64 {
        2.0f64.powi((*bossfight.current_boss().read()).try_into().unwrap())
    }

    fn time_factor(&self) -> f64 {
        let last_rebirth_timestamp = self.last_rebirth_timestamp().read().clone();
        let current_timestamp = self.current_timestamp().read().clone();
        let elapsed_seconds = current_timestamp
            .duration_since(&last_rebirth_timestamp)
            .as_secs_f64();
        let elapsed_hours = elapsed_seconds / 3600.0;
        rebirth_time_factor(elapsed_hours)
    }

    fn rebirth_number<BossfightLens: Copy + Readable<Target = Bossfight>>(
        &self,
        bossfight: Store<Bossfight, BossfightLens>,
    ) -> f64 {
        let boss_factor = self.boss_factor(bossfight);
        let time_factor = self.time_factor();
        let number = boss_factor * time_factor;
        number.max(1.0)
    }
}

/// Above 24 hours: 2, increasing logarithmically with base 24.
/// Between 1 and 24 hours: 1 to 2, increasing slowly by between the fourth and fifth roots.
/// Between 30 minutes and 1 hour: 0.1 to 0.5.
/// Between 10 minutes and 30 minutes: 0.01 to 0.1.
/// Below 10 minutes: up to 0.001.
fn rebirth_time_factor(elapsed_hours: f64) -> f64 {
    if elapsed_hours >= 24.0 {
        elapsed_hours.log(24.0) * 2.0
    } else if elapsed_hours >= 1.0 {
        elapsed_hours.powf(1.0 / 24.0f64.log2())
    } else if elapsed_hours >= 0.5 {
        let fraction = (elapsed_hours - 0.5) * 2.0;
        fraction.sqrt() * 0.4 + 0.1
    } else if elapsed_hours >= 1.0 / 6.0 {
        let fraction = (elapsed_hours - 1.0 / 6.0) * 3.0;
        0.01 * 10.0f64.powf(fraction)
    } else {
        let fraction = elapsed_hours * 6.0;
        0.001 * (fraction + 1.0).log2().max(1e-12)
    }
}

#[component]
pub fn RebirthView() -> Element {
    let mut game = use_context::<Store<Game>>();
    let rebirth = game.rebirth();
    let bossfight = game.bossfight();

    rsx! {
        div { class: "vertical", style: "padding: 10px;",
            h2 { "Rebirth" }
            table {
                tr {
                    td { "Boss factor" }
                    td {
                        "2^{bossfight.current_boss()} = "
                        F64 {
                            number: rebirth.boss_factor(bossfight),
                            format_as_integer: true,
                        }
                    }
                }
                tr {
                    td { "Time factor" }
                    td {
                        F64 { number: rebirth.time_factor() }
                    }
                }
                tr {
                    td { "Number change" }
                    td {
                        F64 { number: rebirth.number() }
                        " -> "
                        F64 { number: rebirth.rebirth_number(bossfight) }
                    }
                }
            }
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

#[cfg(test)]
mod tests {
    use crate::game::rebirth::rebirth_time_factor;

    #[test]
    fn test_rebirth_time_factor() {
        assert!((rebirth_time_factor(117.57550765359) - 3.0).abs() < 1e-6);
        assert!((rebirth_time_factor(48.0) - 2.436208583971).abs() < 1e-6);
        assert!((rebirth_time_factor(24.0) - 2.0).abs() < 1e-6);
        assert!(
            (rebirth_time_factor(23.0) - 1.9815210284664686).abs() < 1e-6,
            "rebirth_time_factor(23.0) = {}",
            rebirth_time_factor(23.0),
        );
        assert!(
            (rebirth_time_factor(2.0) - 1.1632041261809372).abs() < 1e-6,
            "rebirth_time_factor(2.0) = {}",
            rebirth_time_factor(2.0),
        );
        assert!((rebirth_time_factor(1.0) - 1.0).abs() < 1e-6);
        assert!((rebirth_time_factor(0.999999999999) - 0.5).abs() < 1e-6);
        assert!((rebirth_time_factor(0.5) - 0.1).abs() < 1e-6);
        assert!((rebirth_time_factor(0.49999999999) - 0.1).abs() < 1e-6);
        assert!((rebirth_time_factor(1.0 / 6.0 + 0.000001) - 0.01).abs() < 1e-6);
        assert!(
            (rebirth_time_factor(1.0 / 6.0 - 0.000001) - 0.001).abs() < 1e-6,
            "rebirth_time_factor({}) = {}",
            1.0 / 6.0 - 0.000001,
            rebirth_time_factor(1.0 / 6.0 - 0.000001),
        );
        assert!((rebirth_time_factor(0.0) - 1e-15).abs() < 1e-21);
    }
}
