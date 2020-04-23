use std::cell::RefCell;
use std::ops::Deref;

/// A scope. There's usually one scope per thread.
///
/// Note: Usually you don't use this directly. See [`thread_scoped_ref`] (with example)
/// and [`with`] / [`scoped`].
///
/// # Safety
///
/// This struct uses unsafe functionality. When calling the `scope` function a reference to the
/// value is kept as long as the `scope` function is running. The reference to the value
/// is removed when the scope function ends. To make sure the reference is removed even in the
/// case of a panic, there's a cleanup struct that performs cleanup when dropped.
///
/// [`thread_scoped_ref`]: macro.thread_scoped_ref.html
/// [`with`]: fn.with.html
/// [`scoped`]: fn.scoped.html
pub struct Scope<T>(RefCell<Option<*const T>>)
where
    T: ?Sized;

/// Creates a new scope without value.
impl<T> Default for Scope<T>
where
    T: ?Sized,
{
    fn default() -> Self {
        Self(RefCell::new(None))
    }
}

impl<T> Scope<T>
where
    T: ?Sized,
{
    /// Run the given function scoped with given value reference.
    ///
    /// Note: Panicking within the function should be ok, the scope is cleaned up properly.
    #[inline]
    pub fn scoped<TFn, TRet>(&self, value: &T, fun: TFn) -> TRet
    where
        TFn: FnOnce() -> TRet,
    {
        // make sure we always remove the value (even when panicking).
        let mut cleanup_on_drop = CleanupOnDrop {
            scope: Some(self),
            previous_value: self.take(),
        };
        self.set(Some(value));
        let fun_result = fun();
        cleanup_on_drop.cleanup();
        fun_result
    }

    /// Runs the given function with the value from the scope (if there's any). If you're not
    /// inside a scope, there won't be a value (function will receive `None`).
    #[inline]
    pub fn with<TFn, TRet>(&self, fun: TFn) -> TRet
    where
        TFn: FnOnce(Option<&T>) -> TRet,
    {
        let value = self.get();
        fun(value)
    }

    #[inline]
    fn set(&self, value: Option<&T>) {
        *self.0.borrow_mut() = if let Some(value) = value {
            Some(value as *const T)
        } else {
            None
        };
    }

    #[inline]
    fn get(&self) -> Option<&T> {
        let self_borrowed = self.0.borrow();
        if let Some(value) = self_borrowed.deref() {
            Some(unsafe { &*(*value) })
        } else {
            None
        }
    }

    #[inline]
    fn take(&self) -> Option<&T> {
        let mut self_borrowed = self.0.borrow_mut();
        if let Some(taken) = self_borrowed.take() {
            Some(unsafe { &*taken })
        } else {
            None
        }
    }
}

struct CleanupOnDrop<'a, T>
where
    T: ?Sized,
{
    scope: Option<&'a Scope<T>>,
    previous_value: Option<&'a T>,
}

impl<'a, T> CleanupOnDrop<'a, T>
where
    T: ?Sized,
{
    fn cleanup(&mut self) {
        if let Some(scope) = self.scope.take() {
            scope.set(self.previous_value);
        }
    }
}

impl<'a, T> Drop for CleanupOnDrop<'a, T>
where
    T: ?Sized,
{
    fn drop(&mut self) {
        self.cleanup();
    }
}
