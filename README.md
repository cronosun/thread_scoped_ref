# thread_scoped_ref

A library that is similar to a thread local storage but allows to store references / dyn Trait within a scope.
It can be used to 'inject' references (if you don't own the data and Rc/Arc is not possible) into something you
don't control entirely (e.g. a function you provide that gets called by a library you don't own).

It's also very similar to [https://github.com/alexcrichton/scoped-tls](https://github.com/alexcrichton/scoped-tls), with those differences:

 * `thread_scoped_ref` works with traits / unsized types.
 * `thread_scoped_ref` does not panic when calling `ScopedKey::with`, instead calls the closure with `None`.

According to `scoped-tls` there once was something similar in the old rust standard library (quote from `scoped-tls`):

> A Rust library providing the old standard library's `scoped_thread_local!` macro as a library implementation on crates.io.

Example use case:

```

          +----- (set) ---------> &Data <------- (access/read) ----------+
          |                                                              |
+---------+------------+    +--------------------------------------------|-------------+
| Data                 |    | External library                           |             |
| (huge/context/no Rc) |    |                                            |             |
+----------------------+    |                                +-----------+------+      |
                            |              ---- (calls) ---> | Your function    |      |
                            |                                +------------------+      |
                            +----------------------------------------------------------+
```


# More information

  * [crates.io](https://crates.io/crates/thread-scoped-ref)
  * [Documentation](https://docs.rs/thread-scoped-ref)
  * Tests

# Usage

## Cargo

```toml
[dependencies]
thread-scoped-ref = "0"
```

## Example

 ```rust
 use thread_scoped_ref::{thread_scoped_ref, scoped, with};
 use std::collections::HashMap;

 thread_scoped_ref!(SOME_ENV_VALUES, HashMap<String, String>);

 /// It's not possible to pass `&HashMap<String, String>` to this function since it's called
 /// by some library you don't control...
 fn read_env_value() {
   // ... so we read from the static 'SOME_ENV_VALUES'.
   with(&SOME_ENV_VALUES, |maybe_env_values| {
     // don't "unwrap" in reality: Since `maybe_env_values` will be `None` if not
     // called within a scope!
     let env_values = maybe_env_values.unwrap();
     assert_eq!("true", env_values.get("delete_entire_ssd").unwrap());
   });
 }

  /// An external library you don't control or generated code.
 fn external_library(function_ptr : fn()) {
    function_ptr();
 }

 let mut env_values = HashMap::default();
 env_values.insert("delete_entire_ssd".to_string(), "true".to_string());
 // Create a scope. Note: We only need a reference to `env_values` (no move required).
 scoped(&SOME_ENV_VALUES, &env_values, || {
   external_library(read_env_value);
 });
 ```

# License

Licensed under either of

 * Apache License, Version 2.0, ([http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))
 * MIT license ([http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))

at your option.
