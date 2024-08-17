use chrono::{DateTime, NaiveDateTime};

pub fn micros_to_naive_datetime(microseconds: i64) -> NaiveDateTime {
    DateTime::from_timestamp_micros(microseconds)
        .expect("Should be able to convert microseconds to a DateTime and then to a NaiveDateTime.")
        .naive_utc()
}
