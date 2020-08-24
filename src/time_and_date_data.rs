use std::cmp::Ordering;
use std::collections::BTreeSet;
pub use std::convert::From;
use std::fmt;
use std::fmt::Display;
use std::str;
use time;
use String;

use chrono::{FixedOffset, Utc, Weekday};
pub use http::Uri;
use mongodb::bson::Bson;
use serde::de::{Error, Unexpected, Visitor};
use serde::export::Formatter;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TimeData {
    pub page_uri: UriWrapper,
    pub node_data: String,
    pub city_times: BTreeSet<CityData>,
    pub last_updated: String,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CityData {
    pub id: i32,
    pub name: String,
    pub time_string: String,
    pub utc_offset: UtcOffset,
    pub is_dls: bool,
    pub url: String,
    pub sort: Sort,
}

#[derive(Debug, Clone)]
pub struct UriWrapper(Uri);

#[derive(Debug, Clone)]
pub struct DayOfWeek(Weekday);

#[derive(Debug, Clone)]
pub struct UtcOffset(FixedOffset);

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Sort {
    ByName,
    ByOffset,
}

impl TimeData {
    pub fn page_uri(&self) -> &Uri {
        &self.page_uri.0
    }
}

impl UriWrapper {
    pub fn new(uri: Uri) -> UriWrapper {
        UriWrapper(uri)
    }
}

impl Default for UriWrapper {
    #[inline]
    fn default() -> UriWrapper {
        UriWrapper(Uri::default())
    }
}

impl Serialize for UriWrapper {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        Bson::String(self.0.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UriWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct UriWrapperVisitor;

        impl<'de> Visitor<'de> for UriWrapperVisitor {
            type Value = UriWrapper;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("Need an URI")
            }

            fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if s.len() > 0 {
                    Ok(UriWrapper(
                        Uri::from_str(&s).expect(format!("Not an URI: {}", &s).as_str()),
                    ))
                } else {
                    Err(Error::invalid_value(Unexpected::Str(&s), &self))
                }
            }
        }
        deserializer.deserialize_identifier(UriWrapperVisitor)
    }

    fn deserialize_in_place<D>(
        _deserializer: D,
        _place: &mut Self,
    ) -> Result<(), <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl Serialize for UtcOffset {
    fn serialize<S>(&self, serializer: S) -> Result<<S as Serializer>::Ok, <S as Serializer>::Error>
    where
        S: Serializer,
    {
        Bson::String(self.0.to_string()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UtcOffset {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        struct UtcOffsetVisitor;

        impl<'de> Visitor<'de> for UtcOffsetVisitor {
            type Value = UtcOffset;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str("Need an UtcOffset:")
            }

            fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if s.len() > 0 {
                    Ok(UtcOffset(FixedOffset::east(
                        time::UtcOffset::parse(&s.replace(":", ""), "%z")
                            .expect(format!("Not a valid UTC-Offset string: {}", &s).as_str())
                            .as_seconds(),
                    )))
                } else {
                    Err(Error::invalid_value(Unexpected::Str(&s), &self))
                }
            }
        }
        deserializer.deserialize_identifier(UtcOffsetVisitor)
    }

    fn deserialize_in_place<D>(
        _deserializer: D,
        _place: &mut Self,
    ) -> Result<(), <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        unimplemented!()
    }
}

impl Default for Sort {
    fn default() -> Self {
        Self::ByName
    }
}

impl Display for TimeData {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "Scanned Page: {}\nElement Data:{}\nCurrent UTC: {}\nCity Times:\n{}",
            self.page_uri.0,
            self.node_data,
            Utc::now(),
            self.city_times
                .iter()
                .fold(String::new(), |prev, v| prev
                    + format!("\t{}\n", v).as_str())
                .trim_end_matches("\n")
        )
    }
}

impl PartialEq for CityData {
    fn eq(&self, other: &CityData) -> bool {
        self.name == other.name
    }
}

impl Eq for CityData {}

impl PartialOrd for CityData {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for CityData {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.sort {
            Sort::ByName => format!("{}-{}", self.name, self.utc_offset.get()).cmp(&format!(
                "{}-{}",
                other.name,
                other.utc_offset.get()
            )),
            Sort::ByOffset => format!("{}-{}", self.utc_offset.get(), self.name).cmp(&format!(
                "{}-{}",
                other.utc_offset.get(),
                other.name
            )),
        }
    }
}

impl Display for CityData {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(
            fmt,
            "{}",
            format!(
                "City Id {} = {}: {:?}, {}, {}. Url: {}",
                self.id,
                self.name,
                self.utc_offset,
                self.time_string,
                if self.is_dls { "DST" } else { "Winter Time" },
                self.url
            )
            .as_str()
        )
    }
}

impl DayOfWeek {
    pub fn get(&self) -> Weekday {
        self.0
    }
}

impl From<&str> for DayOfWeek {
    fn from(s: &str) -> Self {
        match &*s.to_uppercase() {
            "MON" => DayOfWeek(Weekday::Mon),
            "TUE" => DayOfWeek(Weekday::Tue),
            "WED" => DayOfWeek(Weekday::Wed),
            "THU" => DayOfWeek(Weekday::Thu),
            "FRI" => DayOfWeek(Weekday::Fri),
            "SAT" => DayOfWeek(Weekday::Sat),
            "SUN" => DayOfWeek(Weekday::Sun),
            _ => DayOfWeek(Weekday::Sun),
        }
    }
}

impl From<Weekday> for DayOfWeek {
    fn from(s: Weekday) -> Self {
        DayOfWeek(s)
    }
}

impl Default for DayOfWeek {
    fn default() -> Self {
        DayOfWeek(Weekday::Sun)
    }
}

impl UtcOffset {
    pub fn get(&self) -> FixedOffset {
        self.0
    }
}

impl From<FixedOffset> for UtcOffset {
    fn from(d: FixedOffset) -> Self {
        UtcOffset(d)
    }
}

impl Default for UtcOffset {
    fn default() -> Self {
        UtcOffset(FixedOffset::east(0))
    }
}

#[macro_export]
macro_rules! merge {
    ( $( $more_time_data:expr ),* ) => {
        {
            let mut merged = TimeData::default();
            $(
               merged.city_times = merged.city_times.union(&$more_time_data.unwrap().city_times).cloned().collect();
            )*
           vec!(merged)
        }
    };
}

#[macro_export]
macro_rules! to_vec {
    ( $( $more_time_data:expr ),* ) => {
        {
            let mut vec = Vec::new();
            $(
               vec.push($more_time_data.unwrap());
            )*
            vec
        }
    };
}
