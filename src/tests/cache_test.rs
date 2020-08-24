use std::collections::{BTreeSet, HashMap};

use chrono::{Duration, Utc};

use crate::cache::{cache_invalid, urls_from};
use crate::parse_timeanddate_dot_com::TimeData;

#[test]
fn cache_invalid_test() {
    //No cache, no urls...
    let data = &mut HashMap::default();
    let urls = &mut HashMap::default();

    assert_eq!(cache_invalid(data, urls), true);

    //1 cache; 0 urls: unequal count...
    data.insert(
        String::from("Key1"),
        TimeData {
            last_updated: Utc::now().to_rfc3339(),
            ..Default::default()
        },
    );
    assert_eq!(cache_invalid(data, urls), true);

    //1 cache; 1 urls: equal count, but url-key not found in cache...
    urls.insert(String::from("Key2"), String::from("Url1"));
    assert_eq!(cache_invalid(data, urls), true);

    //1 cache; 2 urls: unequal equal count, but url-key IS found in cache...
    urls.insert(String::from("Key1"), String::from("Url1"));
    assert_eq!(cache_invalid(data, urls), true);

    //2 cache; 2 urls: equal equal count, AND url-keys ARE all found in cache...
    data.insert(
        String::from("Key2"),
        TimeData {
            last_updated: Utc::now().to_rfc3339(),
            ..Default::default()
        },
    );
    assert_eq!(cache_invalid(data, urls), false);

    //3 cache; 2 urls: unequal equal count, AND url-keys ARE all found in cache...
    data.insert(
        String::from("Key3"),
        TimeData {
            last_updated: Utc::now().to_rfc3339(),
            ..Default::default()
        },
    );
    assert_eq!(cache_invalid(data, urls), true);

    //3 cache; 3 urls: equal equal count, AND url-keys ARE all found in cache...
    urls.insert(String::from("Key3"), String::from("Url1"));
    assert_eq!(cache_invalid(data, urls), false);

    //Invalidate cache by making a the last_updated field in a the cached item older than 480 minutes ago
    data.insert(
        String::from("Key3"),
        TimeData {
            last_updated: (Utc::now() - Duration::minutes(8 * 60 + 1)).to_rfc3339(),
            ..Default::default()
        },
    );
    assert_eq!(cache_invalid(data, urls), true);

    //Validate cache by making a the last_updated field in a the cached item one second less than 480 minutes ago
    data.insert(
        String::from("Key3"),
        TimeData {
            last_updated: (Utc::now() - Duration::minutes(8 * 60) + Duration::seconds(1))
                .to_rfc3339(),
            ..Default::default()
        },
    );
    assert_eq!(cache_invalid(data, urls), false);
}

#[test]
fn urls_from_test() {
    assert_eq!(
        urls_from("./src/tests/test-urls.txt")
            .iter()
            .map(|(k, v)| format!("{}={}|", k, v))
            .collect::<BTreeSet<String>>().iter()
            .map(|s| s.as_str())
            .collect::<String>(),
        "Afrika=A|Asia=A|Australasia=A|Europa=E|North Americas=N|Popular Cities=P|South Americas=S|".to_string()
    );
}

#[test]
fn some_urls_commented_out_test() {
    assert_eq!(
        urls_from("./src/tests/test-urls-some-commented-out.txt")
            .iter()
            .map(|(k, v)| format!("{}={}|", k, v))
            .collect::<BTreeSet<String>>()
            .iter()
            .map(|s| s.as_str())
            .collect::<String>(),
        "Afrika=A|Asia=A|Europa=E|North Americas=N|South Americas=S|".to_string()
    );
}
