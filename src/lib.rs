//! # Thread scoped reference
//!
//! A library that is similar to a thread local storage but allows to store references / dyn
//! Trait within a scope.
//!
//! It's similar to `std::thread_local` but allows you to store non-static references. Since the
//! reference is non-static, the value has to be scoped (the reference MUST NOT escape the scope).
//! It also works with dynamic dispatch (e.g. `dyn Trait`). Scopes can be nested. Everything is
//! thread-local.
//!
//! # Example
//!
//! Short example:
//!
//! ```
//! use thread_scoped_ref::{thread_scoped_ref, scoped, with};
//! use std::collections::HashMap;
//!
//! thread_scoped_ref!(SOME_ENV_VALUES, HashMap<String, String>);
//!
//! /// It's not possible to pass `&HashMap<String, String>` to this function since it's called
//! /// by some library you don't control...
//! fn read_env_value() {
//!   // ... so we read from the static 'SOME_ENV_VALUES'.
//!   with(&SOME_ENV_VALUES, |maybe_env_values| {
//!     // don't "unwrap" in reality: Since `maybe_env_values` will be `None` if not
//!     // called within a scope!
//!     let env_values = maybe_env_values.unwrap();
//!     assert_eq!("true", env_values.get("delete_entire_ssd").unwrap());
//!   });
//! }
//!
//!  /// An external library you don't control or generated code.
//! fn external_library(function_ptr : fn()) {
//!    function_ptr();
//! }
//!
//! let mut env_values = HashMap::default();
//! env_values.insert("delete_entire_ssd".to_string(), "true".to_string());
//! // Create a scope. Note: We only need a reference to `env_values` (no move required).
//! scoped(&SOME_ENV_VALUES, &env_values, || {
//!   external_library(read_env_value);
//! });
//! ```
//!
//! Long example:
//!
//! ```
//! use thread_scoped_ref::{thread_scoped_ref, scoped, with};
//!
//! /// Declare the `REF_TO_A_STRING`.
//! thread_scoped_ref!(REF_TO_A_STRING, str);
//!
//! /// This function reads the value and prints the value.
//! fn value_consumer() {
//!   with(&REF_TO_A_STRING, |maybe_string| {
//!     // `maybe_string` is `Some` if this is called within a scope, or `None` if not called
//!     // within a scope.
//!     if let Some(string) = maybe_string {
//!       println!("String is: '{}'", string);
//!     } else {
//!       println!("There's no string.");
//!     }
//!   });
//! }
//!
//! // Example #1: prints `There's no string` (since not called within a scope).
//! value_consumer();
//!
//! // Example #2: With a scope.
//! let my_string = "The String!".to_string();
//! // note: We use the reference and not the actual string. It's not static!
//! let my_string_ref = &my_string;
//! scoped(&REF_TO_A_STRING, my_string_ref, || {
//!   // prints `String is: 'The String!'`
//!   value_consumer();
//! });
//!
//! // Example #3: Nested scopes.
//! let another_string = "Another string".to_string();
//! scoped(&REF_TO_A_STRING, &another_string, || {
//!   // prints `String is: 'Another string'`
//!   value_consumer();
//!   // a nested scope.
//!   scoped(&REF_TO_A_STRING, my_string_ref, || {
//!     // prints `String is: 'The String!'`
//!     value_consumer();
//!   });
//!   // prints `String is: 'Another string'`
//!   value_consumer();
//! });
//!
//! // Example #4: No scope (like example 1). prints `There's no string`.
//! value_consumer();
//! ```
//!
//! # Use case
//!
//! It's useful if you need to 'inject' some sort of context into a function you provide that gets
//! called by a library you don't control. One example is Serde: You can write custom
//! serialize/deserialize methods but it's not possible to call them with custom data (a context) -
//! unless you also write the serialization/deserialization of the container manually (not by using
//! Serde derive).
//!
//! Something like this can be achieved with thread scoped references. See the Serde demo
//! test for details.

mod helper;
mod scope;

pub use helper::*;
pub use scope::Scope;

/// A shortcut macro for `thread_local! { static IDENTIFIER : Scope<Type> = Scope::default() }`.
///
/// # See also
///
///   * [`with`]
///   * [`scoped`]
///
/// # Examples
///
/// With a struct:
///
/// ```
/// use thread_scoped_ref::{thread_scoped_ref, scoped, with};
///
/// struct MyStruct(String);
///
/// thread_scoped_ref!(MY_STRUCT, MyStruct);
///
/// // use it:
/// let demo_struct = MyStruct("Hello".to_string());
///
/// scoped(&MY_STRUCT, &demo_struct, || {
///   with(&MY_STRUCT, |maybe_my_struct_ref| {
///     assert_eq!("Hello", maybe_my_struct_ref.unwrap().0);
///   })
/// })
/// ```
///
/// With a trait / dynamic dispatch (note the `dyn`):
///
/// ```
/// use thread_scoped_ref::{thread_scoped_ref, scoped, with};
///
/// trait MyTrait {
///   fn string(&self) -> &str;
/// }
///
/// struct StructImplementingMyTrait(String);
///
/// impl MyTrait for StructImplementingMyTrait {
///   fn string(&self) -> &str {
///     &self.0
///   }
/// }
///
/// thread_scoped_ref!(MY_TRAIT, dyn MyTrait);
///
/// // use it:
/// let my_struct = StructImplementingMyTrait("Hello World".to_string());
///
/// scoped(&MY_TRAIT, &my_struct, || {
///   with(&MY_TRAIT, |maybe_my_trait_ref| {
///     assert_eq!("Hello World", maybe_my_trait_ref.unwrap().string());
///   })
/// })
/// ```
///
/// [`with`]: fn.with.html
/// [`scoped`]: fn.scoped.html
#[macro_export]
macro_rules! thread_scoped_ref {
    ($identifier:ident, $typ:ty) => {
        std::thread_local! {
            static $identifier: $crate::Scope<$typ> = $crate::Scope::default();
        }
    };
}
