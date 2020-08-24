/// The 'tests'-directory sitting next to the 'src'-directory: Home of the integration tests.
/// In order for this binary crate to run integration tests, both lib.rs AND main.rs have to be present.
/// See more on this topic: https://doc.rust-lang.org/book/ch11-03-test-organization.html#integration-tests-for-binary-crates
mod e2e_tests;
