use crate::Scope;
use std::thread::LocalKey;

/// Execute a function scoped with given value reference.
///
/// See [`thread_scoped_ref`] for an example.
///
/// [`thread_scoped_ref`]: macro.thread_scoped_ref.html
#[inline]
pub fn scoped<T, TFn, TRet>(key: &'static LocalKey<Scope<T>>, value: &T, fun: TFn) -> TRet
where
    TFn: FnOnce() -> TRet,
    T: ?Sized,
{
    key.with(|scope| scope.scoped(value, fun))
}

/// Gets the reference to value from the current scope. Given function will receive
/// `None` if this is not called within a scope.
///
/// See [`thread_scoped_ref`] for an example.
///
/// [`thread_scoped_ref`]: macro.thread_scoped_ref.html
#[inline]
pub fn with<T, TFn, TRet>(key: &'static LocalKey<Scope<T>>, fun: TFn) -> TRet
where
    TFn: FnOnce(Option<&T>) -> TRet,
    T: ?Sized,
{
    key.with(|scope| scope.with(fun))
}
