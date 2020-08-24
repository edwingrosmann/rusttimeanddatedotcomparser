#![warn(warnings)]
#![macro_use]
pub mod animation;
pub mod cache;
pub mod clock_widget;
pub mod druid_clock_app;
pub mod druid_ui;
pub mod mongo_api;
pub mod parse_timeanddate_dot_com;

#[cfg(test)]
mod tests;

/// In order for this binary crate to run integration tests, both lib.rs AND main.rs have to be present.
/// main::main() call this function.
/// See more on this topic: https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests-for-binary-crates
pub fn main(test_mode: bool) {
    druid_ui::show(cache::fetch_time_data(), test_mode);
    druid_clock_app::show(test_mode);
    animation::show_animation(test_mode);
}
