#![warn(rust_2018_idioms)]
#![warn(warnings)]

use chrono::{DateTime, Timelike};

use crate::parse_timeanddate_dot_com::{city_utc_offset, DayOfWeek};

#[test]
fn city_utc_offset_test() {
    //UTC date = 2020-01-03 23:30 +00:00...
    let mut date = DateTime::from(
        DateTime::parse_from_str("2020-01-04 12:30 +13:00", "%Y-%m-%d %H:%M %z").unwrap(),
    );
    println!("UTC Date: {}", date);

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Sun"), 12, 30), &date)
            .get()
            .to_string(),
        String::from("+13:00")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Sun"), 11, 30), &date)
            .get()
            .to_string(),
        String::from("+12:00")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Fri"), 11, 30), &date)
            .get()
            .to_string(),
        String::from("-12:00")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Sun"), 1, 30), &date)
            .get()
            .to_string(),
        String::from("+02:00")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Sun"), 1, 45), &date)
            .get()
            .to_string(),
        String::from("+02:15")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Sun"), 2, 15), &date)
            .get()
            .to_string(),
        String::from("+02:45")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Fri"), 22, 45), &date)
            .get()
            .to_string(),
        String::from("-00:45")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Fri"), 22, 30), &date)
            .get()
            .to_string(),
        String::from("-01:00")
    );
    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Fri"), 23, 15), &date)
            .get()
            .to_string(),
        String::from("-00:15")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Fri"), 22, 15), &date)
            .get()
            .to_string(),
        String::from("-01:15")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Fri"), 22, 46), &date)
            .get()
            .to_string(),
        String::from("-00:44")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Fri"), 23, 15), &date)
            .get()
            .to_string(),
        String::from("-00:15")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Fri"), 23, 45), &date)
            .get()
            .to_string(),
        String::from("+00:15")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Sat"), 1, 15), &date)
            .get()
            .to_string(),
        String::from("+01:45")
    );
    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Sun"), 0, 37), &date)
            .get()
            .to_string(),
        String::from("+01:07")
    );
    date = date.with_hour(2).unwrap();
    println!("Changed UTC Date: {}", date);

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Thu"), 23, 37), &date)
            .get()
            .to_string(),
        String::from("-02:53")
    );
    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Thu"), 20, 28), &date)
            .get()
            .to_string(),
        String::from("-06:02")
    );

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Thu"), 22, 28), &date)
            .get()
            .to_string(),
        String::from("-04:02")
    );

    date = date.with_hour(0).unwrap();
    println!("Changed UTC Date: {}", date);

    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Thu"), 23, 59), &date)
            .get()
            .to_string(),
        String::from("-00:31")
    );
    assert_eq!(
        city_utc_offset(&(DayOfWeek::from("Thu"), 23, 28), &date)
            .get()
            .to_string(),
        String::from("-01:02")
    );
}
