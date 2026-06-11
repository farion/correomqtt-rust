use time::{
    format_description::well_known::Rfc3339, macros::format_description, OffsetDateTime, UtcOffset,
};

const DATE_TIME_FORMAT: &[time::format_description::FormatItem<'_>] =
    format_description!("[year]-[month]-[day] [hour]:[minute]:[second]");
const TIME_FORMAT: &[time::format_description::FormatItem<'_>] =
    format_description!("[hour]:[minute]:[second]");

pub(crate) fn local_date_time(timestamp: &str) -> String {
    format_timestamp(timestamp, DATE_TIME_FORMAT)
}

pub(crate) fn local_time(timestamp: &str) -> String {
    format_timestamp(timestamp, TIME_FORMAT)
}

fn format_timestamp(
    timestamp: &str,
    format: &[time::format_description::FormatItem<'_>],
) -> String {
    let Ok(parsed) = OffsetDateTime::parse(timestamp, &Rfc3339) else {
        return timestamp.to_owned();
    };
    parsed
        .to_offset(local_offset())
        .format(format)
        .unwrap_or_else(|_| timestamp.to_owned())
}

fn local_offset() -> UtcOffset {
    UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC)
}
