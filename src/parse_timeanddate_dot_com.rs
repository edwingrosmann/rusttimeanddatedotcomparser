//#![allow(warnings)]
#![warn(warnings)]
#![warn(rust_2018_idioms)]
#![macro_use]

use core::str::FromStr;
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::HashMap;
use std::convert::From;
use std::rc::Rc;
use std::str;
use std::string::String;

use chrono::{DateTime, Datelike, FixedOffset, Timelike, Utc};
use http::Uri;
use markup5ever::interface::Attribute;
use markup5ever_rcdom::Node;
use markup5ever_rcdom::NodeData;

use local_data::UriWrapper;
pub use local_data::{CityData, DayOfWeek, Sort, TimeData, UtcOffset};
use reader::{fetch_url_body, get_dom, Result};

#[path = "web_page_reader.rs"]
mod reader;

#[path = "time_and_date_data.rs"]
mod local_data;

#[tokio::main]
pub async fn fetch_time_data_from_website(url: String, sort: Sort) -> Result<TimeData> {
    //Fetch the URL's DOM, Create the Time-Data and add UTC as city location;
    //Then let's get this party started, shall we...
    parse_node(
        &get_dom(fetch_url_body(&url).await?).document.children,
        add_utc(&mut create_time_data(&url), sort),
        "",
        &mut CityData {
            sort,
            id: -1,
            ..Default::default()
        },
    )
}

fn parse_node(
    cell: &RefCell<Vec<Rc<Node>>>,
    time_data: &mut TimeData,
    indent: &str,
    city_data: &mut CityData,
) -> Result<TimeData> {
    let children: Vec<Rc<Node>> = cell.clone().into_inner();

    if !children.is_empty() {
        for c in children.iter() {
            process_element(c, time_data, city_data, indent);
            parse_node(
                &c.children,
                time_data,
                (indent.to_owned() + "-").as_str(),
                city_data,
            )?;
        }
    }
    //the node_data-field is for the recursive crawling process; do not chuck it over the fence...
    time_data.node_data = Default::default();
    Ok(time_data.clone())
}

///An example of City-data html
///
/// ```html
/// <td>
///    <a href="/worldclock/new-zealand/auckland">   <-- the City Details URL
///      Auckland                                    <-- The City name
///    </a>  
///    <span id=p26s class=wds> *</span>             <-- the City-id and the DST-astrix
/// </td>
/// <td id=p26 class=rbi>                            <-- the City-id
///    Thu 9:00 p.m.                                 <-- the City's relative time         
/// </td>
/// ```
///
/// The parent element is a <td>
/// It all starts when an <a> is found; the is the city name is in it. </br>
/// Next to it a <span> with an id-attribute: If the child is an astrix, it is DST. </br>
/// Next up is a <td> with a matching id-attribute and it holds the City's time. </br>
fn process_element(c: &Rc<Node>, time_data: &mut TimeData, city_data: &mut CityData, indent: &str) {
    //    Capture the string-data of the current node and its parent...
    time_data.node_data.push_str(&parent_of(
        c,
        to_str(&c.data, (indent.to_string() + ">").as_str()).as_str(),
        indent,
    ));

    if let NodeData::Element { name, attrs, .. } = &c.data {
        let name = name.local.get(..).unwrap();

        //The City-data is tucked away in an <a>...
        if name == "a" {
            city_data.name = get_children_as_flat_string(c);
            city_data.url = format!(
                "{}://{}{}",
                time_data.page_uri().scheme_str().unwrap(),
                time_data.page_uri().host().unwrap(),
                get_attribute::<String>("href", &attrs.borrow()).unwrap()
            );
        } else if let Some(id) = get_city_id::<i32>("id", &attrs.borrow()) {
            //The first 'id' is found in a <span>; capture it so it can be matched in the next iteration...
            if id == city_data.id {
                city_data.time_string = get_children_as_flat_string(c);

                //the time-string needs to be converted into an absolute date-time string; e.g. Thu, 02-01-2020 21:05 +13:00...
                update_city_data(city_data);
                //Now that the city-data has been fully populated, it is to be stored...
                time_data.city_times.insert(city_data.clone());
            } else {
                city_data.id = id;
                city_data.is_dls = get_children_as_flat_string(c).contains("*");
                //With the id already found, it means we have arrived at the <td>;
                //In there is the (relative) city-time; e.g. 'Thu 9:05 p.m.'...
            }
        }
    }
}

fn get_children_as_flat_string(c: &Rc<Node>) -> String {
    c.children
        .clone()
        .into_inner()
        .iter()
        .map(|c| to_str(&c.data, "").replace(".", ""))
        .collect()
}

fn update_city_data(city_data: &mut CityData) {
    //the time string must at least have length 3 so that the day-of-the-week can be determined...
    if city_data.time_string.len() > 2 {
        let utc_time: DateTime<Utc> = DateTime::from(Utc::now());
        let city_time_data = parse_city_time_string(&city_data.time_string);
        city_data.utc_offset = city_utc_offset(&city_time_data, &utc_time);
    }
}

///The returned tuple has elements 1) DayOfWeek, 2) city-hour(24h format),3) city-minutes
fn parse_city_time_string(time: &String) -> (DayOfWeek, i32, i32) {
    let mut tc = time.split_whitespace();
    let day_of_week = DayOfWeek::from(tc.next().unwrap());

    let city_time: Vec<&str> = tc.next().unwrap().split(':').collect();

    //12:04 am has to become 00:04...
    let mut city_hour = city_time[0].parse::<i32>().unwrap() % 12;
    let city_minute = city_time[1].parse::<i32>().unwrap();

    if "PM" == tc.next().unwrap().to_uppercase() {
        city_hour += 12;
    }

    (day_of_week, city_hour, city_minute)
}

pub fn city_utc_offset(
    (city_day_of_week, city_hour, city_minute): &(DayOfWeek, i32, i32),
    utc_time: &DateTime<Utc>,
) -> UtcOffset {
    let utc_day_of_week = utc_time.weekday();

    if city_day_of_week.get() == utc_day_of_week {
        let mut h = city_hour - utc_time.hour() as i32;
        let mut m = city_minute - utc_time.minute() as i32;
        if h < 0 && m > 0 {
            h += 1;
            m = 60 - m;
            //in case a Negative 00:45 is required the sign has to be FORCED: the ':+'-formatter
            //formats a zero as '+00' you see...
            return UtcOffset::from(FixedOffset::west(3600 * h + 60 * m));
        } else if h == 0 && m < 0 {
            //in case a Negative 00:15 is required the sign has to be FORCED: the ':+'-formatter
            //formats a zero as '+00' you see...
            return UtcOffset::from(FixedOffset::west(60 * m.abs()));
        }
        UtcOffset::from(FixedOffset::east(3600 * h + 60 * m))
    } else if city_day_of_week.get().succ() == utc_day_of_week {
        let mut h = utc_time.hour() as i32 + 24 - city_hour;
        let mut m = city_minute - utc_time.minute() as i32;

        if m > 0 {
            h -= 1;
            m = 60 - m;
        } else if m < 0 {
            m = -m;
        };
        UtcOffset::from(FixedOffset::west(3600 * h + 60 * m.abs()))
    } else {
        let mut h = city_hour + 24 - utc_time.hour() as i32;
        let mut m = city_minute - utc_time.minute() as i32;

        if m < 0 {
            h -= 1;
            m = m + 60;
        };

        UtcOffset::from(FixedOffset::east(3600 * h + 60 * m.abs()))
    }
}

fn parent_of(node: &Rc<Node>, name: &str, indent: &str) -> String {
    if let Some(parent) = &node.parent.take().unwrap().upgrade() {
        return format!(
            "Parent: {} has child {}\n",
            to_str(&parent.data, (indent.to_string() + "}").as_str()),
            name
        );
    }
    String::new()
}

fn to_str(c: &NodeData, indent: &str) -> String {
    match &c {
        NodeData::Comment { contents } => {
            format!("Comment : {} {}", indent, contents.get(..).unwrap())
        }
        NodeData::Text { contents } => format!("{}{}", indent, contents.borrow().get(..).unwrap()),
        NodeData::Element {
            name, ref attrs, ..
        } => format!(
            "Element : {} '{}'; attributes: {}",
            indent,
            name.borrow().local.get(..).unwrap(),
            attrs
                .borrow()
                .iter()
                .fold(String::new(), |prev, attr| prev
                    + &attr.name.local.get(..).unwrap()
                    + "="
                    + &attr.value.get(..).unwrap()
                    + ", ")
                .trim_end_matches(", ")
        ),
        _ => format!("Some Node: {} {:?}", indent, c),
    }
}

fn get_attribute<T: FromStr>(attr_name: &str, attrs: &Vec<Attribute>) -> Option<T> {
    for a in attrs.iter() {
        if a.name.local.get(..).unwrap() == attr_name {
            return a.value.get(..).unwrap().parse().ok();
        }
    }
    None
}

fn get_city_id<T: FromStr>(attr_name: &str, attrs: &Vec<Attribute>) -> Option<T> {
    match get_attribute::<String>(attr_name, attrs) {
        Some(id) => id
            .trim_start_matches("p")
            .trim_end_matches("s")
            .parse()
            .ok(),
        _ => None,
    }
}

fn create_time_data(url: &String) -> TimeData {
    TimeData {
        page_uri: UriWrapper::new(url.parse::<Uri>().unwrap()),
        ..Default::default()
    }
}

fn add_utc(data: &mut TimeData, sort: Sort) -> &mut TimeData {
    data.city_times.insert(CityData {
        name: "UTC".to_string(),
        url: String::from("https://www.timeanddate.com/time/aboututc.html"),
        sort: sort,
        id: -1,
        ..Default::default()
    });
    data
}

pub fn download_time_data(sort: Sort, urls: &HashMap<String, String>) -> HashMap<String, TimeData> {
    urls.iter()
        .map(|(k, s)| {
            (
                k.to_string(),
                fetch_time_data_from_website(s.to_string(), sort).unwrap(),
            )
        })
        .collect()
}
