use serde::{de, Deserialize, Deserializer};
use std::sync::atomic::{AtomicU64, Ordering};
use thread_scoped_ref::{scoped, thread_scoped_ref, with};

thread_scoped_ref!(CURRENT_CONTEXT, dyn Context);

/// We can use the convenient Deserialize macro.
#[derive(Deserialize, Eq, PartialEq, Debug)]
struct MyStructure {
    a_string: String,
    a_bool: bool,
    remote_handle: HandleFromContext,
    another_handle: HandleFromContext,
    more: Option<String>,
}

/// This data does not come from Serde, instead it's read from the context.
#[derive(Eq, PartialEq, Debug)]
struct HandleFromContext(u64);

impl<'de> Deserialize<'de> for HandleFromContext {
    fn deserialize<D>(_deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        // here we don't read any data from `deserializer`... instead we take the handles
        // from the context.
        with(&CURRENT_CONTEXT, |maybe_context| {
            if let Some(context) = maybe_context {
                let maybe_handle = context.next_handle();
                if let Some(handle) = maybe_handle {
                    Ok(handle)
                } else {
                    Err(de::Error::custom("No more handles."))
                }
            } else {
                Err(de::Error::custom(
                    "Cannot deserialize HandleFromContext when \
                there's no context in scope.",
                ))
            }
        })
    }
}

trait Context {
    /// Returns next handle ... or `None` if there's no more handle.
    fn next_handle(&self) -> Option<HandleFromContext>;
}

struct ContextImplementation {
    counter: AtomicU64,
}

impl Default for ContextImplementation {
    fn default() -> Self {
        Self {
            counter: AtomicU64::new(33),
        }
    }
}

impl Context for ContextImplementation {
    fn next_handle(&self) -> Option<HandleFromContext> {
        // Of course in reality this would not just return ordered numbers...
        let handle = self.counter.fetch_add(1, Ordering::SeqCst);
        if handle <= 34 {
            Some(HandleFromContext(handle))
        } else {
            None
        }
    }
}

/// This deserializes my structure from a json string. Note: We only have the context as a
/// non-static reference (the context itself is static, but not the reference).
fn deserialize_my_structure_from_json_string(
    context: &(dyn Context + 'static),
    string: &str,
) -> MyStructure {
    // We need to set the context
    scoped(&CURRENT_CONTEXT, context, || {
        serde_json::from_str(&string).unwrap()
    })
}

#[test]
fn test_deserialization() {
    // Note: The handles are not found here.
    let json = r#"
    {
        "a_string" : "My String",
        "a_bool" : true,
        "more" : "Another string"
    }
    "#;
    let context = ContextImplementation::default();
    let my_structure = deserialize_my_structure_from_json_string(&context, json);

    assert_eq!(
        MyStructure {
            a_string: "My String".to_string(),
            a_bool: true,
            remote_handle: HandleFromContext(33),
            another_handle: HandleFromContext(34),
            more: Some("Another string".to_string())
        },
        my_structure
    );
}
