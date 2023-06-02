// Protocol Buffers - Google's data interchange format
// Copyright 2023 Google Inc.  All rights reserved.
// https://developers.google.com/protocol-buffers/
//
// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions are
// met:
//
//     * Redistributions of source code must retain the above copyright
// notice, this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above
// copyright notice, this list of conditions and the following disclaimer
// in the documentation and/or other materials provided with the
// distribution.
//     * Neither the name of Google Inc. nor the names of its
// contributors may be used to endorse or promote products derived from
// this software without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT
// OWNER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL,
// SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
// DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
// THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
// (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
// OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

// copybara:strip_begin
// See http://go/rust-proxy-reference-types for design discussion.
// copybara:strip_end

use std::fmt::Debug;
use std::marker::{Send, Sync};

/// Represents a type that can be accessed through a reference-like proxy.
///
/// Implemented by all Rust types appearing as Protobuf field types.
pub trait Proxied {
    /// Represents shared immutable access to this value through a proxy type.
    type View<'msg>: ViewFor<'msg, Self> + Copy + Send + Sync + Unpin + Sized + Debug
    where
        Self: 'msg;

    /// Represents a unique reference-like mutator proxy for this value.
    type Mut<'msg>: MutFor<'msg, Self> + Sync + Sized + Debug
    where
        Self: 'msg;
}

/// Type aliases for more concise spelling of Proxied associated types.
pub type View<'msg, T> = <T as Proxied>::View<'msg>;
pub type Mut<'msg, T> = <T as Proxied>::Mut<'msg>;

/// Declares operations common to all views.
///
/// This trait is intentionally made non-object-safe to prevent a potential
/// future incompatible change.
pub trait ViewFor<'msg, T: Proxied + ?Sized>
where
    Self: Sized,
{
    /// Coerces the `View` into a shorter lifetime (bound by the lifetime of
    /// self).
    ///
    /// Since `View` is `Copy`, it is not necessary to call this function in
    /// non-generic code. It's there only for generic code.
    fn as_view(&self) -> View<'_, T>;

    /// Coerces the `View` into a shorter lifetime (possibly longer than the
    /// lifetime of self).
    ///
    /// Since `View` is `Copy`, it is not necessary to call this function in
    /// non-generic code. It's there only for generic code.
    fn into_view<'shorter>(self) -> View<'shorter, T>
    where
        'msg: 'shorter;
}

/// Declares operations common to all mutators.
///
/// This trait is intentionally made non-object-safe to prevent a potential
/// future incompatible change.
pub trait MutFor<'msg, T: Proxied + ?Sized>: ViewFor<'msg, T>
where
    Self: Sized,
{
    /// Coerces the `Mut` into a shorter lifetime (bound by the lifetime of
    /// self).
    ///
    /// It is not necessary to call this function in non-generic code. It's
    /// there only for generic code.
    fn as_mut<'shorter: 'msg>(&'shorter mut self) -> Mut<'shorter, T>;

    /// Coerces the `Mut` into a shorter lifetime (possibly longer than the
    /// lifetime of self).
    ///
    /// It is not necessary to call this function in non-generic code. It's
    /// there only for generic code.
    fn into_mut<'shorter>(self) -> Mut<'shorter, T>
    where
        'msg: 'shorter;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, PartialEq)]
    struct MyProxied {
        val: String,
    }

    impl MyProxied {
        fn as_view(&self) -> View<'_, Self> {
            MyProxiedView { my_proxied_ref: self }
        }

        fn as_mut(&mut self) -> Mut<'_, Self> {
            MyProxiedMut { my_proxied_ref: self }
        }
    }

    impl Proxied for MyProxied {
        type View<'msg> = MyProxiedView<'msg>;
        type Mut<'msg> = MyProxiedMut<'msg>;
    }

    #[derive(Debug, Clone, Copy)]
    struct MyProxiedView<'msg> {
        my_proxied_ref: &'msg MyProxied,
    }

    impl MyProxiedView<'_> {
        fn val(&self) -> &str {
            &self.my_proxied_ref.val
        }
    }

    impl<'msg> ViewFor<'msg, MyProxied> for MyProxiedView<'msg> {
        fn as_view(&self) -> View<'msg, MyProxied> {
            *self
        }

        fn into_view<'shorter>(self) -> View<'shorter, MyProxied>
        where
            'msg: 'shorter,
        {
            self
        }
    }

    #[derive(Debug)]
    struct MyProxiedMut<'msg> {
        my_proxied_ref: &'msg mut MyProxied,
    }

    impl MyProxiedMut<'_> {
        fn set_val(&mut self, new_val: String) {
            self.my_proxied_ref.val = new_val;
        }
    }

    impl<'msg> ViewFor<'msg, MyProxied> for MyProxiedMut<'msg> {
        fn as_view(&self) -> View<'_, MyProxied> {
            MyProxiedView { my_proxied_ref: self.my_proxied_ref }
        }
        fn into_view<'shorter>(self) -> View<'shorter, MyProxied>
        where
            'msg: 'shorter,
        {
            MyProxiedView { my_proxied_ref: self.my_proxied_ref }
        }
    }

    impl<'msg> MutFor<'msg, MyProxied> for MyProxiedMut<'msg> {
        fn as_mut<'shorter: 'msg>(&'shorter mut self) -> Mut<'shorter, MyProxied> {
            Self { my_proxied_ref: self.my_proxied_ref }
        }

        fn into_mut<'shorter>(self) -> Mut<'shorter, MyProxied>
        where
            'msg: 'shorter,
        {
            Self { my_proxied_ref: self.my_proxied_ref }
        }
    }

    #[test]
    fn test_view_access() {
        let my_proxied = MyProxied { val: "Hello World".to_string() };

        let my_view = my_proxied.as_view();

        assert_eq!(my_view.val(), my_proxied.val);
    }

    #[test]
    fn test_mut_access() {
        let mut my_proxied = MyProxied { val: "Hello World".to_string() };

        let mut my_mut = my_proxied.as_mut();
        my_mut.set_val("Hello indeed".to_string());

        let val_after_set = my_mut.as_view().val().to_string();
        assert_eq!(my_proxied.val, val_after_set);
        assert_eq!(my_proxied.val, "Hello indeed");
    }

    fn reborrow_mut_into_view<'a>(
        x: Mut<'a, MyProxied>,
        y: View<'a, MyProxied>,
    ) -> [View<'a, MyProxied>; 2] {
        // [x.as_view(), y]` fails to compile with:
        //   `ERROR: attempt to return function-local borrowed content`
        [x.into_view(), y] // OK: we return the same lifetime as we got in.
    }

    #[test]
    fn test_mut_into_view() {
        let mut my_proxied = MyProxied { val: "Hello World".to_string() };
        let other_proxied = MyProxied { val: "Hello2".to_string() };

        reborrow_mut_into_view(my_proxied.as_mut(), other_proxied.as_view());
    }

    fn require_unified_lifetimes<'a>(_x: Mut<'a, MyProxied>, _y: View<'a, MyProxied>) {}

    #[test]
    fn test_require_unified_lifetimes() {
        let mut my_proxied = MyProxied { val: "Hello1".to_string() };
        let my_mut = my_proxied.as_mut();

        {
            let other_proxied = MyProxied { val: "Hello2".to_string() };
            let other_view = other_proxied.as_view();
            require_unified_lifetimes(my_mut, other_view);
        }
    }

    fn reborrow_generic_as_view<'a, 'b, T>(
        x: &'b mut Mut<'a, T>,
        y: &'b View<'a, T>,
    ) -> [View<'b, T>; 2]
    where
        T: Proxied,
        'a: 'b,
    {
        // `[x, y]` fails to compile because `'a` is not the same as `'b` and the `View`
        // lifetime parameter is (conservatively) invariant.
        [x.as_view(), y.as_view()]
    }

    #[test]
    fn test_reborrow_generic_as_view() {
        let mut my_proxied = MyProxied { val: "Hello1".to_string() };
        let mut my_mut = my_proxied.as_mut();
        let my_ref = &mut my_mut;

        {
            let other_proxied = MyProxied { val: "Hello2".to_string() };
            let other_view = other_proxied.as_view();
            reborrow_generic_as_view::<MyProxied>(my_ref, &other_view);
        }
    }

    fn reborrow_generic_view_into_view<'a, 'b, T>(
        x: View<'a, T>,
        y: View<'b, T>,
    ) -> [View<'b, T>; 2]
    where
        T: Proxied,
        'a: 'b,
    {
        // `[x, y]` fails to compile because `'a` is not the same as `'b` and the `View`
        // lifetime parameter is (conservatively) invariant.
        // `[x.as_view(), y]` fails because that borrow cannot outlive `'b`.
        [x.into_view(), y]
    }

    #[test]
    fn test_reborrow_generic_into_view() {
        let my_proxied = MyProxied { val: "Hello1".to_string() };
        let my_view = my_proxied.as_view();

        {
            let other_proxied = MyProxied { val: "Hello2".to_string() };
            let other_view = other_proxied.as_view();
            reborrow_generic_view_into_view::<MyProxied>(my_view, other_view);
        }
    }

    fn reborrow_generic_mut_into_view<'a, 'b, T>(x: Mut<'a, T>, y: View<'b, T>) -> [View<'b, T>; 2]
    where
        T: Proxied,
        'a: 'b,
    {
        [x.into_view(), y]
    }

    #[test]
    fn test_reborrow_generic_mut_into_view() {
        let mut my_proxied = MyProxied { val: "Hello1".to_string() };
        let my_mut = my_proxied.as_mut();

        {
            let other_proxied = MyProxied { val: "Hello2".to_string() };
            let other_view = other_proxied.as_view();
            reborrow_generic_mut_into_view::<MyProxied>(my_mut, other_view);
        }
    }

    fn reborrow_generic_mut_into_mut<'a, 'b, T>(x: Mut<'a, T>, y: Mut<'b, T>) -> [Mut<'b, T>; 2]
    where
        T: Proxied,
        'a: 'b,
    {
        // `[x, y]` fails to compile because `'a` is not the same as `'b` and the `Mut`
        // lifetime parameter is (conservatively) invariant.
        // `[x.as_mut(), y]` fails because that borrow cannot outlive `'b`.
        [x.into_mut(), y]
    }

    #[test]
    fn test_reborrow_generic_mut_into_mut() {
        let mut my_proxied = MyProxied { val: "Hello1".to_string() };
        let my_mut = my_proxied.as_mut();

        {
            let mut other_proxied = MyProxied { val: "Hello2".to_string() };
            let other_mut = other_proxied.as_mut();
            // No need to reborrow, even though lifetime of &other_view is different
            // than the lifetiem of my_ref. Rust references are covariant over their
            // lifetime.
            reborrow_generic_mut_into_mut::<MyProxied>(my_mut, other_mut);
        }
    }
}
