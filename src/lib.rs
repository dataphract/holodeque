//! Array- and slice-backed double-ended queues in 100% safe Rust.
//!
//! This crate provides [`ArrayDeque`] and [`SliceDeque`], fixed-size ring
//! buffers with interfaces similar to the standard library's [`VecDeque`].
//!
//! [`VecDeque`]: https://doc.rust-lang.org/std/collections/struct.VecDeque.html
//!
//! # Example
//!
//! ```
//! use holodeque::{ArrayDeque, CapacityError};
//!
//! const NUM_TASKS: usize = 8;
//!
//! #[derive(Debug, Default, PartialEq, Eq)]
//! struct Task(&'static str);
//!
//! enum Priority {
//!     Low,
//!     High,
//! }
//!
//! fn add_task(
//!     deque: &mut ArrayDeque<Task, NUM_TASKS>,
//!     task: Task,
//!     priority: Priority,
//! ) -> Result<(), CapacityError<Task>> {
//!     match priority {
//!         Priority::Low => deque.push_back(task),
//!         Priority::High => deque.push_front(task),
//!     }
//! }
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//! # (|| -> Result<(), CapacityError<_>> {
//!     let mut tasks: ArrayDeque<Task, NUM_TASKS> = ArrayDeque::new();
//!
//!     add_task(&mut tasks, Task("take hike"), Priority::Low)?;
//!     add_task(&mut tasks, Task("call mom"), Priority::High)?;
//!     add_task(&mut tasks, Task("eat pizza"), Priority::Low)?;
//!
//!     assert_eq!(tasks.pop_front(), Some(Task("call mom")));
//!     assert_eq!(tasks.pop_front(), Some(Task("take hike")));
//!     assert_eq!(tasks.pop_front(), Some(Task("eat pizza")));
//!     assert_eq!(tasks.pop_front(), None);
//!
//!     Ok(())
//! # })().unwrap();
//! # Ok(())
//! }
//! ```
//!
//! # Features
//!
//! - `std`
//!   - Optional, enabled by default
//!   - Disable for `no_std` support
//!   - Provides [`Error`] implementation for [`CapacityError`]
//! - `serde`
//!   - Optional
//!   - Provides:
//!     - [`Serialize`](serde::Serialize) for `ArrayDeque` and `SliceDeque`
//!     - [`Deserialize`](serde::Deserialize) for `ArrayDeque`
//!     - [`DeserializeSeed`](serde::de::DeserializeSeed) for `SliceDeque`
//!
//! [`Error`]: https://doc.rust-lang.org/std/error/trait.Error.html
//!
//! # Safe initialization mechanism
//!
//! The containers provided by `holodeque` use the [`Default`] implementation of
//! an element type to safely initialize unused space. This contrasts with other
//! collection types such as [`arrayvec::ArrayVec`], which represent unused
//! space using [`MaybeUninit`]. This mechanism is borrowed from [`tinyvec`].
//!
//! [`Default`]: https://doc.rust-lang.org/core/default/trait.Default.html
//! [`arrayvec::ArrayVec`]: https://docs.rs/arrayvec
//! [`MaybeUninit`]: https://doc.rust-lang.org/core/mem/union.MaybeUninit.html
//! [`tinyvec`]: https://docs.rs/tinyvec

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]
#![doc(html_root_url = "https://docs.rs/holodeque/0.2.0")]

pub mod array_deque;
mod meta;
pub mod slice_deque;

use core::{fmt, mem};

use crate::meta::{Meta, MetaDrain, MetaLayout};

pub use crate::{array_deque::ArrayDeque, slice_deque::SliceDeque};

/// Provides default implementations for common deque operations.
///
/// This is used to avoid duplicating logic between deque implementations.
pub(crate) trait BaseDeque<T>
where
    T: Default,
{
    type Meta: Meta;

    fn meta(&self) -> &Self::Meta;

    fn meta_mut(&mut self) -> &mut Self::Meta;

    fn items(&self) -> &[T];

    fn items_mut(&mut self) -> &mut [T];

    fn capacity(&self) -> usize;

    #[inline]
    fn len(&self) -> usize {
        self.meta().len()
    }

    fn as_slices(&self) -> (&[T], &[T]) {
        let (front, back) = self.meta().as_ranges();

        (&self.items()[front], &self.items()[back])
    }

    fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        let (high_range, wrap_range) = self.meta().as_ranges();

        if wrap_range.is_empty() {
            // Deque is contiguous.
            return (&mut self.items_mut()[high_range], &mut []);
        }

        let (wrap, front) = self.items_mut().split_at_mut(wrap_range.end);
        let front_range = high_range.start - wrap_range.end..high_range.end - wrap_range.end;

        (&mut front[front_range], &mut wrap[wrap_range])
    }

    #[inline]
    fn is_empty(&self) -> bool {
        matches!(self.meta().layout(), MetaLayout::Empty)
    }

    #[inline]
    fn is_full(&self) -> bool {
        self.len() == self.capacity()
    }

    fn front(&self) -> Option<&T> {
        let front = self.meta().front()?;

        Some(&self.items()[front])
    }

    fn front_mut(&mut self) -> Option<&mut T> {
        let front = self.meta().front()?;

        Some(&mut self.items_mut()[front])
    }

    fn back(&self) -> Option<&T> {
        let back = self.meta().back()?;

        Some(&self.items()[back])
    }

    fn back_mut(&mut self) -> Option<&mut T> {
        let back = self.meta().back()?;

        Some(&mut self.items_mut()[back])
    }

    fn push_front(&mut self, item: T) -> Result<(), CapacityError<T>> {
        match self.meta_mut().reserve_front() {
            Some(front) => {
                self.items_mut()[front] = item;
                Ok(())
            }

            None => Err(CapacityError { item }),
        }
    }

    fn push_back(&mut self, item: T) -> Result<(), CapacityError<T>> {
        match self.meta_mut().reserve_back() {
            Some(back) => {
                self.items_mut()[back] = item;
                Ok(())
            }

            None => Err(CapacityError { item }),
        }
    }

    fn pop_front(&mut self) -> Option<T> {
        let freed = self.meta_mut().free_front()?;

        Some(mem::take(&mut self.items_mut()[freed]))
    }

    fn pop_back(&mut self) -> Option<T> {
        let freed = self.meta_mut().free_back()?;

        Some(mem::take(&mut self.items_mut()[freed]))
    }

    fn clear(&mut self) {
        for freed in self.meta_mut().clear() {
            drop(mem::take(&mut self.items_mut()[freed]));
        }
    }

    fn truncate(&mut self, len: usize) {
        let n = self.len().saturating_sub(len);

        if let Some(drain) = self.meta_mut().drain_back(n) {
            for freed in drain {
                drop(mem::take(&mut self.items_mut()[freed]));
            }
        }
    }
}

/// An immutable iterator over a deque.
pub(crate) struct DequeIter<'a, D, T>
where
    D: BaseDeque<T>,
    T: Default,
{
    meta: D::Meta,
    deque: &'a D,
}

impl<'a, D, T> DequeIter<'a, D, T>
where
    D: BaseDeque<T>,
    T: Default,
{
    pub fn new(deque: &'a D) -> DequeIter<'a, D, T> {
        DequeIter {
            meta: deque.meta().clone(),
            deque,
        }
    }
}

impl<'a, D, T> Iterator for DequeIter<'a, D, T>
where
    D: BaseDeque<T>,
    T: Default + 'a,
{
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.meta.free_front()?;

        Some(&self.deque.items()[next])
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.meta.len(), Some(self.meta.len()))
    }
}

impl<'a, D, T> DoubleEndedIterator for DequeIter<'a, D, T>
where
    D: BaseDeque<T>,
    T: Default + 'a,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        let next_back = self.meta.free_back()?;

        Some(&self.deque.items()[next_back])
    }
}

/// A draining iterator over a deque.
pub(crate) struct DequeDrain<'a, D, T>
where
    D: BaseDeque<T>,
    T: Default,
{
    meta: MetaDrain<D::Meta>,
    deque: &'a mut D,
}

impl<'a, D, T> DequeDrain<'a, D, T>
where
    D: BaseDeque<T>,
    T: Default,
{
    fn front(deque: &'a mut D, n: usize) -> Option<DequeDrain<'a, D, T>> {
        let meta = deque.meta_mut().drain_front(n)?;

        Some(DequeDrain { meta, deque })
    }

    fn back(deque: &'a mut D, n: usize) -> Option<DequeDrain<'a, D, T>> {
        let meta = deque.meta_mut().drain_back(n)?;

        Some(DequeDrain { meta, deque })
    }
}

impl<'a, D, T> Iterator for DequeDrain<'a, D, T>
where
    D: BaseDeque<T>,
    T: Default,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let next = self.meta.next()?;

        Some(mem::take(&mut self.deque.items_mut()[next]))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.meta.size_hint()
    }
}

impl<'a, D, T> Drop for DequeDrain<'a, D, T>
where
    D: BaseDeque<T>,
    T: Default,
{
    fn drop(&mut self) {
        for index in &mut self.meta {
            drop(mem::take(&mut self.deque.items_mut()[index]))
        }
    }
}

/// An error that occurs when attempting to add an item to a deque which is
/// already full.
#[derive(Debug)]
pub struct CapacityError<T> {
    item: T,
}

impl<T> CapacityError<T> {
    /// Returns a reference to the contained value.
    pub fn get(&self) -> &T {
        &self.item
    }

    /// Consumes the error, returning the contained value.
    pub fn into_inner(self) -> T {
        self.item
    }
}

impl<T> fmt::Display for CapacityError<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("deque capacity exceeded")
    }
}

#[cfg(feature = "std")]
impl<T> std::error::Error for CapacityError<T> where T: fmt::Debug {}

pub(crate) enum DequeEnd {
    Front,
    Back,
}
