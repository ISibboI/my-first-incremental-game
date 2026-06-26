use dioxus::prelude::*;
use num::{BigUint, ToPrimitive};

static SUFFIX_TABLE: &[&str] = &[
    "",
    "Thousand",
    "Million",
    "Billion",
    "Trillion",
    "Quadrillion",
    "Quintillion",
    "Sextillion",
    "Septillion",
    "Octillion",
    "Nonillion",
    "Decillion",
    "Undecillion",
    "Duodecillion",
    "Tredecillion",
    "Quattuordecillion",
    "Quindecillion",
    "Sexdecillion",
    "Septendecillion",
    "Octodecillion",
    "Novemdecillion",
    "Vigintillion",
    "Unvigintillion",
    "Duovigintillion",
    "Trevigintillion",
    "Quattorvigintillion",
    "Quinvigintillion",
    "Sexvigintillion",
    "Septemvigintillion",
    "Octovigintillion",
    "Novemvigintillion",
    "Trigintillion",
    "Untrigintillion",
    "Duotrigintillion",
    "Tretrigintillion",
    "Quattortrigintillion",
    "Quintrigintillion",
    "Sextrigintillion",
    "Septentrigintillion",
    "Octotrigintillion",
    "Novemtrigintillion",
    "Quadragintillion",
    "Unquadragintillion",
    "Duoquadragintillion",
    "Trequadragintillion",
    "Quattorquadragintillion",
    "Quinquadragintillion",
    "Sexquadragintillion",
    "Septenquadragintillion",
    "Octoquadragintillion",
    "Novemquadragintillion",
    "Quinquagintillion",
    "Unquinquagintillion",
    "Duoquinquagintillion",
    "Trequinquagintillion",
    "Quattorquinquagintillion",
    "Quinquinquagintillion",
    "Sexquinquagintillion",
    "Septenquinquagintillion",
    "Octoquinquagintillion",
    "Novemquinquagintillion",
    "Sexagintillion",
    "Unsexagintillion",
    "Duosexagintillion",
    "Tresexagintillion",
    "Quattorsexagintillion",
    "Quinsexagintillion",
    "Sexsexagintillion",
    "Septensexagintillion",
    "Octosexagintillion",
    "Novemsexagintillion",
    "Septuagintillion",
    "Unseptuagintillion",
    "Duoseptuagintillion",
    "Treseptuagintillion",
    "Quattorseptuagintillion",
    "Quinseptuagintillion",
    "Sexseptuagintillion",
    "Septenseptuagintillion",
    "Octoseptuagintillion",
    "Novemseptuagintillion",
    "Octogintillion",
    "Unoctogintillion",
    "Duooctogintillion",
    "Treoctogintillion",
    "Quattoroctogintillion",
    "Quinoctogintillion",
    "Sexoctogintillion",
    "Septenoctogintillion",
    "Octooctogintillion",
    "Novemoctogintillion",
    "Nonagintillion",
    "Unnonagintillion",
    "Duononagintillion",
    "Trenonagintillion",
    "Quattuornonagintillion",
    "Quinnonagintillion",
    "Sexnonagintillion",
    "Septennonagintillion",
    "Octononagintillion",
    "Novemnonagintillion",
    "Centillion",
    "Decicentillion",
    "Viginticentillion",
    "Trigintacentillion",
    "Quadragintacentillion",
    "Quinquagintacentillion",
    "Sexagintacentillion",
    "Septuagintacentillion",
    "Octogintacentillion",
    "Nonagintacentillion",
    "Ducentillion",
    "Trucentillion",
    "Quadrigentillion",
    "Quingentillion",
    "Sescentillion",
    "Septingentillion",
    "Octingentillion",
    "Nongentillion",
    "Nonagintanongentillion",
    "Novemnonagintanongentillion",
    "Millinillion",
];

pub fn format_f64(number: f64) -> String {
    if number.is_nan() {
        "NaN".to_string()
    } else if number.is_infinite() {
        if number.is_sign_positive() {
            "Inf".to_string()
        } else {
            "-Inf".to_string()
        }
    } else if number.is_subnormal() || number == 0.0 || number == -0.0 {
        "0".to_string()
    } else if number.abs() < 1.0 {
        format!("{:.3}", number)
    } else {
        let sign = if number < 0.0 { "-" } else { "" };
        let number = number.abs();
        let log10 = number.log10();
        let index = (log10 / 3.0).floor() as usize;
        let display_number = number / 10f64.powi((index * 3) as i32);

        if display_number >= 100.0 {
            format!("{}{:.0} {}", sign, display_number, SUFFIX_TABLE[index])
        } else if display_number >= 10.0 {
            format!("{}{:.1} {}", sign, display_number, SUFFIX_TABLE[index])
        } else {
            format!("{}{:.2} {}", sign, display_number, SUFFIX_TABLE[index])
        }
    }
}

#[component]
pub fn F64(number: ReadSignal<f64>) -> Element {
    let number = *number.read();
    let number = format_f64(number);

    rsx! { "{number}" }
}

#[component]
pub fn U64(number: ReadSignal<u64>) -> Element {
    let number = *number.read();
    let number = format_f64(number as f64);

    rsx! { "{number}" }
}

#[component]
pub fn U32(number: ReadSignal<u32>) -> Element {
    let number = *number.read();
    let number = format_f64(number as f64);

    rsx! { "{number}" }
}

#[component]
pub fn FormatBigUint(number: ReadSignal<BigUint>) -> Element {
    let number = number.read().clone();
    let number = format_f64(number.to_f64().unwrap_or(f64::INFINITY));

    rsx! { "{number}" }
}
