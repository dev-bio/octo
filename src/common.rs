use chrono::{

    Duration as ChronoDuration,
    DateTime as ChronoDateTime,
    Utc as ChronoUtc,
};

pub type Duration = ChronoDuration;
pub type Date = ChronoDateTime<ChronoUtc>;