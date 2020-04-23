use thread_scoped_ref::{scoped, thread_scoped_ref, with};

pub struct Person {
    pub name: String,
}

impl WithName for Person {
    fn name(&self) -> &str {
        &self.name
    }
}

pub struct Thing {
    pub name: String,
}

impl WithName for Thing {
    fn name(&self) -> &str {
        &self.name
    }
}

pub trait WithName {
    fn name(&self) -> &str;
}

thread_scoped_ref!(WITH_NAME, dyn WithName);

#[test]
pub fn simple_test() {
    let mut found_name = None;

    let thing = Thing {
        name: "A red car".to_string(),
    };
    scoped(&WITH_NAME, &thing, || {
        found_name = with(&WITH_NAME, |maybe_with_name| {
            Some(maybe_with_name.unwrap().name().to_string())
        });
    });

    assert_eq!(found_name.unwrap(), "A red car");
}

#[test]
pub fn initially_no_value() {
    let found = with(&WITH_NAME, |maybe_with_name| maybe_with_name.is_some());
    assert_eq!(false, found);
}

/// Outside scopes, there's no value.
#[test]
pub fn no_scope_no_value() {
    // within scope
    let mut found_name = None;
    let thing = Thing {
        name: "A blue car".to_string(),
    };
    scoped(&WITH_NAME, &thing, || {
        found_name = with(&WITH_NAME, |maybe_with_name| {
            Some(maybe_with_name.unwrap().name().to_string())
        });
    });
    assert_eq!(found_name.unwrap(), "A blue car");

    // outside scope (no value).
    let found = with(&WITH_NAME, |maybe_with_name| maybe_with_name.is_some());
    assert_eq!(false, found);
}

/// Scopes can be nested.
#[test]
pub fn nested_scopes() {
    let thing_outer_scope = Thing {
        name: "A green car".to_string(),
    };
    let person_inner_scope = Person {
        name: "Albert".to_string(),
    };

    scoped(&WITH_NAME, &thing_outer_scope, || {
        with(&WITH_NAME, |maybe_with_name| {
            // value is still there
            assert_eq!("A green car", maybe_with_name.unwrap().name());
        });

        // inner scope
        scoped(&WITH_NAME, &person_inner_scope, || {
            with(&WITH_NAME, |maybe_with_name| {
                // value is still there
                assert_eq!("Albert", maybe_with_name.unwrap().name());
            });
        });

        // Now there's again the value from the outer scope.
        with(&WITH_NAME, |maybe_with_name| {
            assert_eq!("A green car", maybe_with_name.unwrap().name());
        });
    });
}

/// Using scope manually. You usually don't need this, see the `simple_test` test.
#[test]
pub fn simple_test_manual() {
    let mut found_name = None;

    WITH_NAME.with(|scope| {
        let person = Person {
            name: "Person One".to_string(),
        };
        scope.scoped(&person, || {
            WITH_NAME.with(|scope| {
                scope.with(|maybe_with_name| {
                    found_name = Some(maybe_with_name.unwrap().name().to_string());
                })
            });
        });
    });

    assert_eq!(found_name.unwrap(), "Person One");
}

thread_scoped_ref!(PERSON, Person);

/// Also works with structures (must not be a trait).
#[test]
pub fn test_with_struct() {
    let mut found_name = None;

    let person = Person {
        name: "Einstein".to_string(),
    };
    scoped(&PERSON, &person, || {
        found_name = with(&PERSON, |maybe_person| {
            Some(maybe_person.unwrap().name.clone())
        });
    });

    assert_eq!(found_name.unwrap(), "Einstein");
}
