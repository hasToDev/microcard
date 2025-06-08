use async_graphql_derive::SimpleObject;
use linera_sdk::linera_base_types::Amount;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Deserialize, Eq, Ord, PartialOrd, PartialEq, Serialize, SimpleObject)]
pub struct ChipSet {
    pub min_bet: Amount,
    pub max_bet: Amount,
    pub denominations: [Amount; 5],
    pub denominations_str: [String; 5],
    pub enabled: [bool; 5],
}

pub fn calculate_chip_set(balance: Amount) -> ChipSet {
    // Minimum base value (smallest chip)
    let mut base = Amount::from_tokens(100);

    // Handle balances below the minimum
    if balance < base {
        let empty_str = [String::new(), String::new(), String::new(), String::new(), String::new()];
        return ChipSet {
            min_bet: Amount::ZERO,
            max_bet: balance,
            denominations: [Amount::ZERO; 5],
            denominations_str: empty_str,
            enabled: [false; 5],
        };
    }

    // Calculate the appropriate base level
    while balance >= base.try_mul(500).unwrap_or(Amount::MAX) {
        let next_base = base.try_mul(10).unwrap_or(Amount::ZERO);
        if next_base.is_zero() {
            break;
        } else {
            base = next_base;
        }
    }

    // Generate chip denominations
    let denominations = [
        base,                     // 1x
        base.saturating_mul(5),   // 5x
        base.saturating_mul(25),  // 25x
        base.saturating_mul(100), // 100x
        base.saturating_mul(250), // 250x
    ];

    // Determine enabled status based on balance
    let enabled = [
        denominations[0] <= balance,
        denominations[1] <= balance,
        denominations[2] <= balance,
        denominations[3] <= balance,
        denominations[4] <= balance,
    ];

    // Generate chip String denominations
    let denominations_str = [
        format_units(denominations[0].saturating_div(Amount::ONE)),
        format_units(denominations[1].saturating_div(Amount::ONE)),
        format_units(denominations[2].saturating_div(Amount::ONE)),
        format_units(denominations[3].saturating_div(Amount::ONE)),
        format_units(denominations[4].saturating_div(Amount::ONE)),
    ];

    ChipSet {
        min_bet: denominations[0], // Smallest denomination
        max_bet: balance,          // Player's full balance
        denominations,
        denominations_str,
        enabled,
    }
}

pub fn format_units(value: u128) -> String {
    if value < 1000 {
        return value.to_string();
    }

    const SUFFIXES: [(&str, u128); 11] = [
        ("D", 1_000_000_000_000_000_000_000_000_000_000_000), // 10^33
        ("N", 1_000_000_000_000_000_000_000_000_000_000),     // 10^30
        ("O", 1_000_000_000_000_000_000_000_000_000),         // 10^27
        ("Sp", 1_000_000_000_000_000_000_000_000),            // 10^24
        ("S", 1_000_000_000_000_000_000_000),                 // 10^21
        ("Qi", 1_000_000_000_000_000_000),                    // 10^18
        ("Q", 1_000_000_000_000_000),                         // 10^15
        ("T", 1_000_000_000_000),                             // 10^12
        ("B", 1_000_000_000),                                 // 10^9
        ("M", 1_000_000),                                     // 10^6
        ("K", 1_000),                                         // 10^3
    ];

    for &(suffix, divisor) in SUFFIXES.iter() {
        if value >= divisor {
            let scaled = value as f64 / divisor as f64;
            // Handle values that round up to 1000
            if scaled >= 999.95 {
                // Try next higher suffix
                if let Some(&(next_suffix, next_divisor)) = SUFFIXES.get(SUFFIXES.len() - SUFFIXES.iter().position(|&s| s.1 == divisor).unwrap() - 1) {
                    let next_scaled = value as f64 / next_divisor as f64;
                    return format_float(next_scaled, next_suffix);
                }
            }
            return format_float(scaled, suffix);
        }
    }

    value.to_string()
}

fn format_float(value: f64, suffix: &str) -> String {
    // Round to the nearest tenth
    let rounded = (value * 10.0).round() / 10.0;

    if rounded.fract() == 0.0 {
        format!("{:.0}{}", rounded, suffix)
    } else {
        let s = format!("{:.1}", rounded);
        s.trim_end_matches('0').trim_end_matches('.').to_string() + suffix
    }
}
