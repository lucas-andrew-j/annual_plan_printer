use std::env;
use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use time::{Date, OffsetDateTime, PrimitiveDateTime, Time, UtcOffset, Weekday};
use time::Month;

struct RRule {
    freq: String,
    until: String,
    count: u16,
    by_month: Month,
    by_day_n: u8,
    by_day_weekday: Weekday,
}

struct TimeZone {
    id: String,
    std_offset: UtcOffset,
    dst_offset: UtcOffset,
    dst_start: RRule,
    dst_end: RRule,
}

impl TimeZone {
    fn get_tz_offset<'a>(date: Date, time: Time, local: bool, tz: TimeZone) -> Result<UtcOffset, &'a str> {
        let checking_date = OffsetDateTime::new_in_offset(date, time, UtcOffset::from_hms(0, 0, 0)?);

        //Find out which RRule was the most recent
        let dst_start_datetime = OffsetDateTime::new_in_offset(
            get_nth_weekday(tz.dst_start.by_day_n, date.year(), tz.dst_start.by_month, tz.dst_start.by_day_weekday),
            Time::from_hms(2, 0, 0)?,
            if local {
                UtcOffset::from_hms(0, 0, 0)?
            } else {
                tz.std_offset
            }
        );

        if checking_date < dst_start_datetime {
            return Ok(tz.std_offset)
        }

        let dst_end_datetime = OffsetDateTime::new_in_offset(
            get_nth_weekday(tz.dst_end.by_day_n, date.year(), tz.dst_end.by_month, tz.dst_end.by_day_weekday),
            Time::from_hms(2, 0, 0)?,
            if local {
                UtcOffset::from_hms(0, 0, 0)?
            } else {
                tz.std_offset
            }
        );

        if checking_date < dst_end_datetime {
            return Ok(tz.dst_offset)
        }

        Ok(tz.std_offset)

        //if it's after 2 AM local time

            //Need to hash out the details about how this transition happens at 2 AM
            //From UTC and from Pacific Time, at both DST transitions
            //When DST is ending, if the Std time calculated from UTC is 2 AM, the transition has occurred. Put in STD.
            //When DST is ending, if the local time is 2 AM or later, the transition has occurred. Use STD offset to get UTC.
            //When DST is starting, if the STD time calculated from UTC is 2 AM or later, make it 3 AM (Put in DST).
            //When DST is starting, local times between >= 0200 and <0300 will move forward one hour. Before is STD, after is DST.
        //Get the corresponding offset
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let file = File::open(args.get(1).expect("Input file not specified"));
}

fn offset_date_time_from_ical(ical_str: &str) -> OffsetDateTime {
    let mut start = 0;
    let mut local_time = false;

    if ical_str[0..4].eq("TZID") {
        local_time = true;
        for c in ical_str.chars() {
            start += 1;
            if c == ':' {
                break;
            }
        }
    } else if ical_str.chars().nth(15) != Some('Z') {
        panic!("Invalid date time format.");
    }

    let year = ical_str[start..start + 4].parse::<i32>().expect("Invalid date time format");
    let month = ical_str[start + 4..start + 6].parse::<u8>().expect("Invalid date time format");
    let month = match month {
        1 => Month::January,
        2 => Month::February,
        3 => Month::March,
        4 => Month::April,
        5 => Month::May,
        6 => Month::June,
        7 => Month::July,
        8 => Month::August,
        9 => Month::September,
        10 => Month::October,
        11 => Month::November,
        12 => Month::December,
        _ => panic!("Invalid month value")
    };
    let day = ical_str[start + 6..start + 8].parse::<u8>().expect("Invalid date time format");

    let hour = ical_str[start + 9..start + 11].parse::<u8>().expect("Invalid date time format");
    let minute = ical_str[start + 11..start + 13].parse::<u8>().expect("Invalid date time format");
    let second = ical_str[start + 13..start + 15].parse::<u8>().expect("Invalid date time format");

    let date = Date::from_calendar_date(year, month, day).expect("Invalid date values");
    let time = Time::from_hms(hour, minute, second).expect("Invalid time values");

    let offset =
    if local_time {
        UtcOffset::from_hms(0, 0, 0).expect("Error processing offset")
        //TODO Get the TZID and find the matching offset to go with it. Use that.
    } else {
        UtcOffset::from_hms(0, 0, 0).expect("Error processing offset")
    };

    OffsetDateTime::new_in_offset(date, time, offset)
}

fn get_nth_weekday(n: u8, year: i32, month: Month, wkday: Weekday) -> Date {
    let mut first = Date::from_calendar_date(year, month, 1)
        .expect("Error parsing year and month for nth weekday.");
    let mut i = 0;
    while first.weekday().nth_next(i) != wkday {
        i += 1;
    }

    first.replace_day(i + 1 + (n - 1) * 7).expect("Error calculating day for nth weekday.")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ical_datetime_utc_to_pst() {
        let dt = offset_date_time_from_ical("20241103T120000Z");

        assert!(dt.year() == 2024);
        assert!(dt.month() == Month::November);
        assert!(dt.day() == 3);
        assert!(dt.hour() == 4);
        assert!(dt.minute() == 00);
        assert!(dt.second() == 00);
    }

    #[test]
    fn ical_datetime_utc_to_pdt() {
        let dt = offset_date_time_from_ical("20240310T120000Z");

        assert!(dt.year() == 2024);
        assert!(dt.month() == Month::March);
        assert!(dt.day() == 10);
        assert!(dt.hour() == 5);
        assert!(dt.minute() == 00);
        assert!(dt.second() == 00);
    }

    #[test]
    fn ical_datetime_pst() {
        let dt = offset_date_time_from_ical("TZID=America/Los_Angeles:20240228T221518");

        assert!(dt.year() == 2024);
        assert!(dt.month() == Month::February);
        assert!(dt.day() == 28);
        assert!(dt.hour() == 22);
        assert!(dt.minute() == 15);
        assert!(dt.second() == 18);
    }

    #[test]
    fn ical_datetime_pdt() {
        let dt = offset_date_time_from_ical("TZID=America/Los_Angeles:20240828T221518");

        assert!(dt.year() == 2024);
        assert!(dt.month() == Month::August);
        assert!(dt.day() == 28);
        assert!(dt.hour() == 22);
        assert!(dt.minute() == 15);
        assert!(dt.second() == 18);
    }

    #[test]
    fn nth_weekday() {
        assert_eq!(get_nth_weekday(1, 2024, Month::September, Weekday::Sunday),
                   Date::from_calendar_date(2024, Month::September, 1).unwrap());
        assert_eq!(get_nth_weekday(5, 2024, Month::September, Weekday::Monday),
                   Date::from_calendar_date(2024, Month::September, 30).unwrap());
    }

    #[test]
    fn get_tz_offset_pst() {
        todo!()
    }

    #[test]
    fn get_tz_offset_pdt() {
        todo!()
    }

    #[test]
    fn get_tz_offset_utc_during_pst() {
        todo!()
    }

    #[test]
    fn get_tz_offset_utc_during_pdt() {
        todo!()
    }
}