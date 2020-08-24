use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{prelude::*, BufReader};
use std::str::FromStr;

use chrono::{DateTime, Duration, FixedOffset, Utc};

use crate::mongo_api;
use crate::parse_timeanddate_dot_com::{download_time_data, Sort, TimeData};

/// This Cache-feature assumes a LOCAL MONGODB.
/// Storing City Data in MongoDb but only if the 'cache' program argument has been set.
/// Also an optional ttl=480 can be provided: this sets the Time To Live of the stored (cached) data  in [minutes]
pub fn fetch_time_data() -> HashMap<String, TimeData> {
    //The Urls to download the time-data from...
    let urls = &urls_from("urls.txt");

    //Is the time-data to be stored in a MongoDb?...
    if do_cache() {
        return use_cache(urls);
    }
    download_time_data(Sort::ByName, urls)
}

fn use_cache(urls: &HashMap<String, String>) -> HashMap<String, TimeData> {
    let mut map = mongo_api::load_stored_time_data();

    // Is there cached data, does it have the same document count as the number of requested web-pages
    // or has it expired?...
    if cache_invalid(&map, &urls) {
        println!("Refreshing Cache: downloading all time-data now...");
        map = download_time_data(Sort::ByName, urls);
        mongo_api::replace_stored_data_with(&map);
    } else {
        println!("Serving up time-data from cache...");
    }
    map
}

///The cache is deemed invalid when:
/// 1) There IS NO Cached-data
/// 2) The number of cached items does NOT match the number of urls
/// 3) NOT All urls-keys are present in the cache
/// 4) ANY cached-item has expired.
///So When there are six URLs and six cached items but the keys do not ALL match, this function will return false.
pub fn cache_invalid(
    cached_data: &HashMap<String, TimeData>,
    urls: &HashMap<String, String>,
) -> bool {
    let last_updated = oldest_last_updated(cached_data);
    let cache_time_to_live = ttl();

    //The cache-validation criteria...
    let cache_size_invalid = cached_data.len() == 0 || cached_data.len() != urls.len();
    let cache_incomplete = urls
        .iter()
        .filter(|(key, _url)| !cached_data.contains_key(*key))
        .count()
        != 0;
    let cache_expired = Utc::now() - last_updated > cache_time_to_live;

    //Share the love...
    println!(
        "Cache age = {}; TTL (Time To Live) = {}; Cache expired: {}.  Cache contain correct number of elements: {}. All URLs have been cached: {}.",
        (Utc::now() - last_updated), cache_time_to_live, cache_expired,
        !cache_size_invalid, !cache_incomplete
    );

    //Return the verdict...
    cache_size_invalid || cache_incomplete || cache_expired
}

///Commented-out urls  - line starts '//' or '#' - are ignored
pub fn urls_from(file: &str) -> HashMap<String, String> {
    let mut urls = HashMap::new();
    for l in BufReader::new(File::open(file).expect(&format!("Please place file '{}' in the same directory as the executable; it contains the URLS to download the time-data from", file))).lines() {
        let u = l.unwrap();
        if can_use(&u) {
            let d: Vec<&str> = u.splitn(2, "=").collect();
            if d.len() == 2 && d[1].len() > 0 {
                urls.insert(d[0].trim().to_string(), d[1].trim().to_string());
            }
        }
    }
    urls
}

fn can_use(u: &String) -> bool {
    u.trim().len() != 0 && !u.trim().starts_with("#") && !u.trim().starts_with("//")
}

fn oldest_last_updated(map: &HashMap<String, TimeData>) -> DateTime<Utc> {
    let mut oldest = Utc::now();

    //find oldest 'last_updated' date-time...
    for td in map.values() {
        let t = DateTime::<FixedOffset>::parse_from_rfc3339(&td.last_updated)
            .unwrap()
            .into();
        if oldest > t {
            oldest = t;
        }
    }
    oldest
}

fn do_cache() -> bool {
    env::args().collect::<String>().contains("cache")
}

fn ttl() -> Duration {
    let mut ttl = 8 * 60;

    for attr in env::args().collect::<Vec<String>>() {
        if attr.contains("ttl") {
            ttl = i64::from_str(attr.split("=").last().unwrap()).expect(
                "Please provide the Time-To-Live-in-minutes-parameter e.g. as follows: ttl=480",
            )
        }
    }
    Duration::minutes(ttl)
}
