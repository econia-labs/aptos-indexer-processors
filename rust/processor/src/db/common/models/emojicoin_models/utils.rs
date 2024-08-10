use chrono::{DateTime, NaiveDateTime};

pub fn micros_to_naive_datetime(microseconds: i64, field_name: &str) -> NaiveDateTime {
    DateTime::from_timestamp_micros(microseconds)
        .expect(
            format!(
                "{} must have a valid timestamp in microseconds.",
                field_name
            )
            .as_str(),
        )
        .naive_utc()
}
