use chrono::{DateTime, NaiveDateTime};

pub fn micros_to_naive_datetime(microseconds: i64) -> NaiveDateTime {
    DateTime::from_timestamp_micros(microseconds)
        .expect(
            format!("Failed to convert {microseconds} as microseconds to a timestamp.").as_str(),
        )
        .naive_utc()
}
