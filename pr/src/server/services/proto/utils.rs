use chrono::{DateTime, NaiveDateTime, Utc};

pub fn convert_timestamp(millis: i64) -> DateTime<Utc> {
    let seconds = (millis / 1000) as i64;
    let nanos = ((millis % 1000) * 1_000_000) as u32;
    DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(seconds, nanos), Utc)
}
