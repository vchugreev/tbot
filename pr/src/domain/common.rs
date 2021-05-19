use chrono::{DateTime, Utc};

pub trait Received {
    fn received(&self) -> DateTime<Utc>;
}
