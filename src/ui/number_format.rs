use dioxus::prelude::*;
use num::{BigUint, ToPrimitive};
use thousands::Separable;

static SUFFIX_TABLE: &[&str] = &[
    "",
    "",
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

pub fn format_f64(number: f64, format_as_integer: bool) -> String {
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
        if format_as_integer {
            format!("{:.0}", number)
        } else {
            let sign = if number < 0.0 { "-" } else { "" };
            let number = number.abs();
            let log10 = number.log10().floor();
            let precision = (-log10 + 2.0) as usize;

            let result = format!("{}{:.precision$}", sign, number, precision = precision);
            let significant_digit_count = result
                .chars()
                .filter(|c| c.is_ascii_digit())
                .skip_while(|c| *c == '0')
                .count();
            if significant_digit_count > 3 && result.ends_with('0') {
                result[..result.len() - 1].to_string()
            } else {
                result
            }
        }
    } else {
        let sign = if number < 0.0 { "-" } else { "" };
        let number = number.abs();
        let log10 = number.log10();
        let index = (log10 / 3.0).floor() as usize;
        let display_number = number / 10f64.powi((index * 3) as i32);

        if index == 0 && format_as_integer {
            format!("{}{}", sign, display_number.round().separate_with_commas())
        } else if index == 1 {
            let display_number = display_number * 1e3;
            let display_number_string = display_number.round().separate_with_commas();
            if display_number_string == 1e6.separate_with_commas() {
                format!(
                    "{sign}{:.2} {}",
                    display_number / 1e6,
                    SUFFIX_TABLE[index + 1]
                )
            } else {
                format!("{sign}{display_number_string}")
            }
        } else {
            let (display_number_string, index) = if display_number >= 100.0 {
                let display_number_string = display_number.round().separate_with_commas();
                if display_number_string.len() == 3 || index == 0 {
                    (display_number_string, index)
                } else {
                    (format!("{:.2}", display_number / 1e3), index + 1)
                }
            } else if display_number >= 10.0 {
                let mut display_number_string = format!("{display_number:.1}");
                if display_number_string.len() > 4 {
                    display_number_string.pop(); // '0'
                    display_number_string.pop(); // '.'
                }
                (display_number_string, index)
            } else {
                let mut display_number_string = format!("{display_number:.2}");
                if display_number_string.len() > 4 {
                    display_number_string.pop(); // '0'
                }
                (display_number_string, index)
            };

            format!(
                "{sign}{display_number_string}{}{}",
                if SUFFIX_TABLE[index].is_empty() {
                    ""
                } else {
                    " "
                },
                SUFFIX_TABLE[index],
            )
        }
    }
}

pub fn format_u64(number: u64) -> String {
    if number < 1_000_000 {
        return number.separate_with_commas();
    }

    let log10 = number.ilog10();
    let index = (log10 / 3) as usize;
    let display_number = number as f64 / 10_f64.powi((index * 3) as i32);

    if display_number >= 100.0 {
        format!("{display_number:.0} {}", SUFFIX_TABLE[index])
    } else if display_number >= 10.0 {
        format!("{display_number:.1} {}", SUFFIX_TABLE[index])
    } else {
        format!("{display_number:.2} {}", SUFFIX_TABLE[index])
    }
}

pub fn format_biguint(number: &BigUint) -> String {
    format_f64(number.to_f64().unwrap_or(f64::INFINITY), true)
}

#[component]
pub fn F64(number: ReadSignal<f64>, format_as_integer: ReadSignal<Option<bool>>) -> Element {
    let number = *number.read();
    let format_as_integer = format_as_integer.read().unwrap_or(false);
    let number = format_f64(number, format_as_integer);

    rsx! { "{number}" }
}

#[component]
pub fn U64(number: ReadSignal<u64>) -> Element {
    let number = *number.read();
    let number = format_u64(number);

    rsx! { "{number}" }
}

#[component]
pub fn U32(number: ReadSignal<u32>) -> Element {
    let number = *number.read();
    let number = format_u64(number.into());

    rsx! { "{number}" }
}

#[component]
pub fn FormatBigUint(number: ReadSignal<BigUint>) -> Element {
    let number = number.read();
    let number = format_biguint(&number);

    rsx! { "{number}" }
}

#[cfg(test)]
mod tests {
    use crate::ui::number_format::format_f64;

    #[test]
    fn test_format_f64() {
        assert_eq!(format_f64(1000000000.0, false), "1.00 Billion");
        assert_eq!(format_f64(999999999.999, false), "1.00 Billion");
        assert_eq!(format_f64(100000000.0, false), "100 Million");
        assert_eq!(format_f64(99999999.999, false), "100 Million");
        assert_eq!(format_f64(10000000.0, false), "10.0 Million");
        assert_eq!(format_f64(9999999.999, false), "10.0 Million");
        assert_eq!(format_f64(1000000.0, false), "1.00 Million");
        assert_eq!(format_f64(999999.999, false), "1.00 Million");
        assert_eq!(format_f64(100000.0, false), "100,000");
        assert_eq!(format_f64(99999.999, false), "100,000");
        assert_eq!(format_f64(10000.0, false), "10,000");
        assert_eq!(format_f64(9999.999, false), "10,000");
        assert_eq!(format_f64(1000.0, false), "1,000");
        assert_eq!(format_f64(999.999, false), "1,000");
        assert_eq!(format_f64(100.0, false), "100");
        assert_eq!(format_f64(99.999, false), "100");
        assert_eq!(format_f64(10.0, false), "10.0");
        assert_eq!(format_f64(9.999, false), "10.0");
        assert_eq!(format_f64(1.0, false), "1.00");
        assert_eq!(format_f64(0.99999, false), "1.00");
        assert_eq!(format_f64(0.999, false), "0.999");
        assert_eq!(format_f64(0.9911, false), "0.991");
        assert_eq!(format_f64(0.99, false), "0.990");
        assert_eq!(format_f64(0.0099, false), "0.00990");
        assert_eq!(format_f64(0.0009999, false), "0.00100");

        assert_eq!(format_f64(-1000000000.0, false), "-1.00 Billion");
        assert_eq!(format_f64(-999999999.999, false), "-1.00 Billion");
        assert_eq!(format_f64(-100000000.0, false), "-100 Million");
        assert_eq!(format_f64(-99999999.999, false), "-100 Million");
        assert_eq!(format_f64(-10000000.0, false), "-10.0 Million");
        assert_eq!(format_f64(-9999999.999, false), "-10.0 Million");
        assert_eq!(format_f64(-1000000.0, false), "-1.00 Million");
        assert_eq!(format_f64(-999999.999, false), "-1.00 Million");
        assert_eq!(format_f64(-100000.0, false), "-100,000");
        assert_eq!(format_f64(-99999.999, false), "-100,000");
        assert_eq!(format_f64(-10000.0, false), "-10,000");
        assert_eq!(format_f64(-9999.999, false), "-10,000");
        assert_eq!(format_f64(-1000.0, false), "-1,000");
        assert_eq!(format_f64(-999.999, false), "-1,000");
        assert_eq!(format_f64(-100.0, false), "-100");
        assert_eq!(format_f64(-99.999, false), "-100");
        assert_eq!(format_f64(-10.0, false), "-10.0");
        assert_eq!(format_f64(-9.999, false), "-10.0");
        assert_eq!(format_f64(-1.0, false), "-1.00");
        assert_eq!(format_f64(-0.99999, false), "-1.00");
        assert_eq!(format_f64(-0.999, false), "-0.999");
        assert_eq!(format_f64(-0.9911, false), "-0.991");
        assert_eq!(format_f64(-0.99, false), "-0.990");
        assert_eq!(format_f64(-0.0099, false), "-0.00990");
        assert_eq!(format_f64(-0.0009999, false), "-0.00100");
    }
}
