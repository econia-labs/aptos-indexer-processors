use chrono::{DateTime, NaiveDateTime};

pub fn micros_to_naive_datetime(microseconds: i64) -> NaiveDateTime {
    DateTime::from_timestamp_micros(microseconds)
        .expect("Should be able to convert microseconds to a DateTime and then to a NaiveDateTime.")
        .naive_utc()
}

pub fn within_past_day(time: NaiveDateTime) -> bool {
    let one_day_ago = chrono::Utc::now() - chrono::Duration::hours(24);

    time.and_utc() > one_day_ago
}

// Removes all leading zeros from an address, adds an "0x" prefix, and converts the address to lowercase.
// Note this does nearly the opposite of what `standardize_address` does, since it removes leading zeros,
// while `standardize_address` pads addresses with leading zeros to 64 characters.
pub fn normalize_address(s: &str) -> String {
    let res = s
        .strip_prefix("0x")
        .unwrap_or(s)
        .trim_start_matches('0')
        .to_lowercase();
    format!("0x{}", res)
}

#[cfg(test)]
mod utils_tests {
    use super::*;

    #[test]
    fn test_strip_leading_zeros() {
        assert_eq!(normalize_address("0x"), "0x");
        assert_eq!(normalize_address("0x0123"), "0x123");
        assert_eq!(normalize_address("0x00123"), "0x123");
        assert_eq!(
            normalize_address("0x000000000000000000000000000000000000000000000000123"),
            "0x123"
        );
        assert_eq!(
            normalize_address("0x0000000000000000000000000000000000000000000000000000000000000001"),
            "0x1"
        );
        assert_eq!(
            normalize_address("0x0000000000000000000000000000000000000000000000000000000000000002"),
            "0x2"
        );
        assert_eq!(
            normalize_address("0x0000000000000000000000000000000000000000000000000000000000000003"),
            "0x3"
        );
        assert_eq!(
            normalize_address("0x000000000000000000000000000000000000000000000000000000000000000a"),
            "0xa"
        );
        assert_eq!(
            normalize_address("0x000000000000000000000000000000000000000000000000000000000000000b"),
            "0xb"
        );
        assert_eq!(
            normalize_address("0x000000000000000000000000000000000000000000000000000000000000000c"),
            "0xc"
        );
        assert_eq!(
            normalize_address("0x000000000000000000000000000000000000000000000000000000000000000f"),
            "0xf"
        );
    }

    #[test]
    fn test_upper_leading_zeros() {
        assert_eq!(normalize_address("0x001ABC23"), "0x1abc23");
        assert_eq!(
            normalize_address("0x0000000000000000000000000000000000000000000000000000000000000001"),
            "0x1"
        );
        assert_eq!(
            normalize_address("0x000000000000000000000000000000000000000000000000000000000000000A"),
            "0xa"
        );
        assert_eq!(
            normalize_address("0x000000000000000000000000000000000000000000000000000000000000000C"),
            "0xc"
        );
        assert_eq!(
            normalize_address("0x000000000000000000000000000000000000000000000000000000000000000F"),
            "0xf"
        );
        assert_eq!(
            normalize_address("0x000000000001000000A000000000B00000c000000000000D00000e000000000F"),
            "0x1000000a000000000b00000c000000000000d00000e000000000f"
        );
    }
}
