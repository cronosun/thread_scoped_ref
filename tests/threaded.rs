use std::thread;
use thread_scoped_ref::{scoped, thread_scoped_ref, with};

pub struct SomeValue(usize);

thread_scoped_ref!(SOME_VALUE, SomeValue);

#[test]
pub fn threaded_test() {
    let number_of_threads = 40;
    let number_of_iterations_per_thread = 20000;

    let mut handles = Vec::with_capacity(number_of_threads);
    for thread_num in 0..number_of_threads {
        let handle = thread::spawn(move || {
            for index in 0..number_of_iterations_per_thread {
                let expected_value = thread_num * index;
                let some_value = SomeValue(expected_value);
                let result: Result<(), &'static str> = scoped(&SOME_VALUE, &some_value, || {
                    with(&SOME_VALUE, |maybe_some_value_ref| {
                        if let Some(some_value_ref) = maybe_some_value_ref {
                            if some_value_ref.0 != expected_value {
                                Err("Got incorrect value value")
                            } else {
                                Ok(())
                            }
                        } else {
                            Err("Missing some value")
                        }
                    })
                });
                if let Err(err) = result {
                    return Err(err);
                }
            }
            Ok(())
        });
        handles.push(handle);
    }

    // make sure there's no error.
    for handle in handles {
        let result: Result<(), &'static str> = handle.join().unwrap();
        result.unwrap();
    }
}
