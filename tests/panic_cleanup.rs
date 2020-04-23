use std::panic;
use thread_scoped_ref::{scoped, thread_scoped_ref, with};

pub struct MyValue(String);

thread_scoped_ref!(MY_VALUE, MyValue);

/// It's important that the scoped value does not escape the scope. Even in the case of a panic
/// within the scope the scope has to be reset.
#[test]
pub fn panic_within_scope_and_cleanup() {
    let value = MyValue("The value".to_string());

    let result = panic::catch_unwind(|| {
        scoped(&MY_VALUE, &value, || {
            // so far so good... the value is there.
            with(&MY_VALUE, |maybe_value| {
                assert_eq!(maybe_value.unwrap().0, "The value")
            });
            // now here we panic within the scope...
            panic!("shit happens!");
        });
    });

    // make sure we really did panic.
    assert!(result.is_err());

    // The important part (!!): make sure the scope has been cleaned up properly.
    with(&MY_VALUE, |maybe_value| {
        assert_eq!(true, maybe_value.is_none())
    });
}
