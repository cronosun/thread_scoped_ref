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
//! It's useful if you need to 'inject' some sort of context into a function you can't modify. One
//! example is Serde - the serialize/deserialize methods don't provide a possibility to supply
//! a custom context; with this library this becomes possible. It would be no problem if you
//! owned the context - but if you only have a reference to the context it gets harder.
//!
//! For example, say you have some sort of messaging system (IPC) and the client gets this:
//!
//! ```rust,ignore
//! struct Handle(u32);
//!
//! /// This is the message the client receives from the server.
//! struct Message {
//!   payload : Vec<u8>,
//!   handles : Vec<Handle>,
//! }
//! ```
//!
//! Note that the handles are transferred independently (since the IPC-system has to inspect them;
//! validate and maybe transform). Now say we want to deserialize something like this:
//!
//! ```rust,ignore
//! #[derive(Deserialize)]
//! struct TheStruct {
//!   // this data is from `payload: Vec<u8>` (nothing special here)
//!   description : String,
//!   // this data is from `payload: Vec<u8>` (nothing special here)
//!   extended : bool,
//!   // !!!: This data comes from `handles: Vec<Handle>` and not from `payload: Vec<u8>`
//!   master_handle : Handle,
//!   // !!!: This data comes from `handles: Vec<Handle>` and not from `payload: Vec<u8>`
//!   slave_handle : Handle,
//!   // this data is from `payload: Vec<u8>` (nothing special here)
//!   path : Vec<String>
//! }
//! ```
//!
//! Something like this can be achieved with the thread scoped references. See the Serde demo
//! tests for details.

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
