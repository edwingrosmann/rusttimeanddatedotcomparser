use rusttimeanddatedotcomparser as r;
use rusttimeanddatedotcomparser::parse_timeanddate_dot_com::TimeData;
use std::collections::HashMap;

#[test]
fn main_test() {
    r::main(true);
}
//#[test]
//fn load_clock_app_test() {
//    r::druid_clock_app::show();
//}
//#[test]
//fn druid_ui_test() {
//    r::druid_ui::show(r::cache::fetch_time_data());
//}
#[test]
fn fetch_time_data_test() {
    //clear the cache...
    r::mongo_api::replace_stored_data_with(&HashMap::<String, TimeData>::new());

    //Fresh time-data will now be downloaded and cached (when 'cache' parameter has been provided)...
    assert_eq!(r::cache::fetch_time_data().len(), 7);

    //Fresh time-data will now loaded from cache (when 'cache' parameter has been provided)...
    assert_eq!(r::cache::fetch_time_data().len(), 7);
}
