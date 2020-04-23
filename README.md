# thread_scoped_ref

A library that is similar to a thread local storage but allows to store references / dyn Trait within a scope.
It can be used to 'inject' references (if you don't own the data) into functions you don't control (e.g. functions from an external library).

# Cargo

```toml
[dependencies]
thread-scoped-ref = "0"
```

# More information

See crates.io, the documentation and the tests.

# Example

 ```rust
 use thread_scoped_ref::{thread_scoped_ref, scoped, with};

 /// Declare the `REF_TO_A_STRING`.
 thread_scoped_ref!(REF_TO_A_STRING, str);

 /// This function reads the value and prints the value. This function is usually called by an external
 /// library you don't control.
 fn value_consumer() {
   with(&REF_TO_A_STRING, |maybe_string| {
     // `maybe_string` is `Some` if this is called within a scope, or `None` if not called
     // within a scope.
     if let Some(string) = maybe_string {
       println!("String is: '{}'", string);
     } else {
       println!("There's no string.");
     }
   });
 }

 // Example #1: prints `There's no string` (since not called within a scope).
 value_consumer();

 // Example #2: With a scope.
 let my_string = "The String!".to_string();
 // note: We use the reference and not the actual string. It's not static!
 let my_string_ref = &my_string;
 scoped(&REF_TO_A_STRING, my_string_ref, || {
   // prints `String is: 'The String!'`
   value_consumer();
 });

 // Example #3: Nested scopes.
 let another_string = "Another string".to_string();
 scoped(&REF_TO_A_STRING, &another_string, || {
   // prints `String is: 'Another string'`
   value_consumer();
   // a nested scope.
   scoped(&REF_TO_A_STRING, my_string_ref, || {
     // prints `String is: 'The String!'`
     value_consumer();
   });
   // prints `String is: 'Another string'`
   value_consumer();
 });

 // Example #4: No scope (like example 1). prints `There's no string`.
 value_consumer();
 ```

# License

MIT OR Apache-2.0
