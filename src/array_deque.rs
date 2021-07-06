//! A double-ended queue with fixed capacity, backed by an array.

use crate::{
    meta::{Meta, MetaLayout},
    BaseDeque, CapacityError, DequeDrain, DequeIter,
};

#[derive(Clone, Debug)]
pub(crate) struct ArrayMeta<const N: usize> {
    layout: MetaLayout,
}

impl<const N: usize> Meta for ArrayMeta<N> {
    #[inline(always)]
    fn capacity(&self) -> usize {
        N
    }

    #[inline(always)]
    fn layout(&self) -> MetaLayout {
        self.layout
    }

    #[inline(always)]
    fn set_layout(&mut self, layout: MetaLayout) {
        self.layout = layout;
    }
}

/// A double-ended queue with fixed capacity, backed by an array.
///
/// The capacity of the deque is determined by the generic parameter `N`.
///
/// All values are stored inline; that is, the size of of `ArrayDeque<T, N>` is
/// *at least* `size_of::<[T; N]>()`, regardless of the number of elements
/// currently stored in the deque.
#[derive(Clone, Debug)]
pub struct ArrayDeque<T, const N: usize>
where
    T: Default,
{
    meta: ArrayMeta<N>,
    items: [T; N],
}

impl<T, const N: usize> BaseDeque<T> for ArrayDeque<T, N>
where
    T: Default,
{
    type Meta = ArrayMeta<N>;

    #[inline(always)]
    fn meta(&self) -> &Self::Meta {
        &self.meta
    }

    #[inline(always)]
    fn meta_mut(&mut self) -> &mut Self::Meta {
        &mut self.meta
    }

    #[inline(always)]
    fn items(&self) -> &[T] {
        &self.items
    }

    #[inline(always)]
    fn items_mut(&mut self) -> &mut [T] {
        &mut self.items
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        N
    }
}

impl<T, const N: usize> Default for ArrayDeque<T, N>
where
    T: Default,
{
    #[inline(always)]
    fn default() -> Self {
        Self::new()
    }
}

impl<T, const N: usize> PartialEq for ArrayDeque<T, N>
where
    T: PartialEq + Default,
{
    fn eq(&self, other: &Self) -> bool {
        let mut it_other = other.iter();

        for item_self in self.iter() {
            let item_other = match it_other.next() {
                Some(x) => x,
                None => return false,
            };

            if item_self != item_other {
                return false;
            }
        }

        it_other.next().is_none()
    }
}

impl<T, const N: usize> Eq for ArrayDeque<T, N> where T: PartialEq + Default {}

impl<T, const N: usize> ArrayDeque<T, N>
where
    T: Default,
{
    /// Constructs a new, empty `ArrayDeque<T, N>`.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::ArrayDeque;
    /// # fn main() {
    /// let mut deque: ArrayDeque<String, 8> = ArrayDeque::new();
    ///
    /// assert!(deque.is_empty());
    /// # }
    /// ```
    pub fn new() -> Self {
        ArrayDeque {
            meta: ArrayMeta {
                layout: MetaLayout::Empty,
            },
            items: [(); N].map(|_| Default::default()),
        }
    }

    /// Returns the maximum number of elements the deque may hold.
    ///
    /// This has the same value as the const generic parameter `N`.
    ///
    /// # Example
    ///
    /// ```
    /// use holodeque::ArrayDeque;
    /// # fn main() {
    /// let mut deque: ArrayDeque<(), 42> = ArrayDeque::new();
    ///
    /// assert_eq!(deque.capacity(), 42);
    /// # }
    /// ```
    #[inline]
    pub fn capacity(&self) -> usize {
        BaseDeque::capacity(self)
    }

    /// Returns the number of elements in the deque.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<char, 3> = ArrayDeque::new();
    ///
    /// deque.push_back('a')?;
    /// deque.push_back('b')?;
    /// deque.push_back('c')?;
    ///
    /// assert_eq!(deque.len(), 3);
    /// # Ok(())
    /// # })().unwrap()
    /// # }
    /// ```
    #[inline]
    pub fn len(&self) -> usize {
        BaseDeque::len(self)
    }

    /// Returns `true` if the deque contains no elements.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<u32, 8> = ArrayDeque::new();
    ///
    /// deque.push_back(42)?;
    /// assert!(!deque.is_empty());
    /// deque.pop_front();
    /// assert!(deque.is_empty());
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn is_empty(&self) -> bool {
        BaseDeque::is_empty(self)
    }

    /// Returns `true` if the deque is at capacity.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main()  {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<u32, 4> = ArrayDeque::new();
    ///
    /// deque.push_back(1)?;
    /// deque.push_back(2)?;
    /// deque.push_back(3)?;
    /// assert!(!deque.is_full());
    ///
    /// deque.push_back(4)?;
    /// assert!(deque.is_full());
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn is_full(&self) -> bool {
        BaseDeque::is_full(self)
    }

    /// Returns a reference to the first element in the deque.
    ///
    /// If the deque is empty, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<char, 3> = ArrayDeque::new();
    ///
    /// deque.push_back('a')?;
    /// deque.push_back('b')?;
    /// deque.push_back('c')?;
    ///
    /// assert_eq!(deque.front(), Some(&'a'));
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn front(&self) -> Option<&T> {
        BaseDeque::front(self)
    }

    /// Returns a mutable reference to the first element in the deque.
    ///
    /// If the deque is empty, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<&str, 4> = ArrayDeque::new();
    ///
    /// deque.push_front("old")?;
    /// deque.front_mut().map(|mut val| {
    ///     *val = "new";
    /// });
    ///
    /// assert_eq!(deque.front(), Some(&"new"));
    ///
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn front_mut(&mut self) -> Option<&mut T> {
        BaseDeque::front_mut(self)
    }

    /// Returns a reference to the last element in the deque.
    ///
    /// If the deque is empty, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<char, 3> = ArrayDeque::new();
    ///
    /// deque.push_back('a')?;
    /// deque.push_back('b')?;
    /// deque.push_back('c')?;
    ///
    /// assert_eq!(deque.back(), Some(&'c'));
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn back(&self) -> Option<&T> {
        BaseDeque::back(self)
    }

    /// Returns a mutable reference to the last element in the deque.
    ///
    /// If the deque is empty, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<&str, 4> = ArrayDeque::new();
    ///
    /// deque.push_back("old")?;
    /// deque.back_mut().map(|mut val| {
    ///     *val = "new";
    /// });
    ///
    /// assert_eq!(deque.back(), Some(&"new"));
    ///
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn back_mut(&mut self) -> Option<&mut T> {
        BaseDeque::back_mut(self)
    }

    /// Prepends an element to the deque.
    ///
    /// If the deque is at capacity, an `Err` containing the pushed value is
    /// returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<u32, 3> = ArrayDeque::new();
    ///
    /// deque.push_front(1)?;
    /// deque.push_front(2)?;
    /// deque.push_front(3)?;
    ///
    /// assert_eq!(deque.front(), Some(&3));
    /// assert_eq!(deque.back(), Some(&1));
    ///
    /// // Another element would exceed capacity, so this fails.
    /// let err = deque.push_front(4).unwrap_err();
    /// assert_eq!(err.into_inner(), 4);
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn push_front(&mut self, item: T) -> Result<(), CapacityError<T>> {
        BaseDeque::push_front(self, item)
    }

    /// Appends an element to the deque.
    ///
    /// If the deque is at capacity, an `Err` containing the pushed value is
    /// returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<u32, 3> = ArrayDeque::new();
    ///
    /// deque.push_back(1)?;
    /// deque.push_back(2)?;
    /// deque.push_back(3)?;
    ///
    /// assert_eq!(deque.front(), Some(&1));
    /// assert_eq!(deque.back(), Some(&3));
    ///
    /// // Another element would exceed capacity, so this fails.
    /// let err = deque.push_front(4).unwrap_err();
    /// assert_eq!(err.into_inner(), 4);
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn push_back(&mut self, item: T) -> Result<(), CapacityError<T>> {
        BaseDeque::push_back(self, item)
    }

    /// Removes and returns the first element of the deque.
    ///
    /// If the deque is empty, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<u32, 3> = ArrayDeque::new();
    ///
    /// deque.push_back(1)?;
    /// deque.push_back(2)?;
    /// deque.push_back(3)?;
    ///
    /// assert_eq!(deque.pop_front(), Some(1));
    /// assert_eq!(deque.pop_front(), Some(2));
    /// assert_eq!(deque.pop_front(), Some(3));
    /// assert_eq!(deque.pop_front(), None);
    ///
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn pop_front(&mut self) -> Option<T> {
        BaseDeque::pop_front(self)
    }

    /// Removes and returns the last element of the deque.
    ///
    /// If the deque is empty, `None` is returned.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<u32, 3> = ArrayDeque::new();
    ///
    /// deque.push_back(1)?;
    /// deque.push_back(2)?;
    /// deque.push_back(3)?;
    ///
    /// assert_eq!(deque.pop_back(), Some(3));
    /// assert_eq!(deque.pop_back(), Some(2));
    /// assert_eq!(deque.pop_back(), Some(1));
    /// assert_eq!(deque.pop_back(), None);
    ///
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn pop_back(&mut self) -> Option<T> {
        BaseDeque::pop_back(self)
    }

    /// Returns an iterator over the elements of the deque.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<&str, 5> = ArrayDeque::new();
    ///
    /// deque.push_back("ideas")?;
    /// deque.push_front("green")?;
    /// deque.push_back("sleep")?;
    /// deque.push_front("colorless")?;
    /// deque.push_back("furiously")?;
    ///
    /// let sentence = deque.iter().cloned().collect::<Vec<_>>();
    ///
    /// assert_eq!(
    ///     sentence,
    ///     &["colorless", "green", "ideas", "sleep", "furiously"],
    /// );
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn iter(&self) -> Iter<'_, T, N> {
        Iter::new(self)
    }

    /// Drains `n` elements from the front of the deque.
    ///
    /// If `n` exceeds `self.len()`, `None` is returned.
    ///
    /// When this method is called, `n` elements are immediately removed from
    /// the front of the deque. If the returned iterator is dropped before
    /// yielding all its items, they are dropped along with it.
    ///
    /// If the returned iterator is leaked (e.g. with [`mem::forget`]), the
    /// drained elements will not be dropped immediately. They may be dropped as
    /// a result of subsequent operations on the deque; otherwise, they will be
    /// dropped when the deque itself is dropped.
    ///
    /// [`mem::forget`]: https://doc.rust-lang.org/stable/core/mem/fn.forget.html
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<u32, 5> = ArrayDeque::new();
    ///
    /// deque.push_back(0)?;
    /// deque.push_back(1)?;
    /// deque.push_back(2)?;
    /// deque.push_back(3)?;
    /// deque.push_back(4)?;
    ///
    /// let mut drain = deque.drain_front(3).unwrap();
    ///
    /// assert_eq!(drain.next(), Some(0));
    /// assert_eq!(drain.next(), Some(1));
    /// assert_eq!(drain.next(), Some(2));
    /// assert_eq!(drain.next(), None);
    /// drop(drain);
    ///
    /// assert_eq!(deque.len(), 2);
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn drain_front(&mut self, n: usize) -> Option<DrainFront<'_, T, N>> {
        DrainFront::new(self, n)
    }

    /// Drains `n` elements from the back of the deque.
    ///
    /// If `n` exceeds `self.len()`, `None` is returned.
    ///
    /// When this method is called, `n` elements are immediately removed from
    /// the back of the deque. If the returned iterator is dropped before
    /// yielding all its items, they are dropped along with it.
    ///
    /// If the returned iterator is leaked (e.g. with [`mem::forget`]), the
    /// drained elements will not be dropped immediately. They may be dropped as
    /// a result of subsequent operations on the deque; otherwise, they will be
    /// dropped when the deque itself is dropped.
    ///
    /// [`mem::forget`]: https://doc.rust-lang.org/stable/core/mem/fn.forget.html
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<u32, 5> = ArrayDeque::new();
    ///
    /// deque.push_back(0)?;
    /// deque.push_back(1)?;
    /// deque.push_back(2)?;
    /// deque.push_back(3)?;
    /// deque.push_back(4)?;
    ///
    /// let mut drain = deque.drain_back(3).unwrap();
    ///
    /// assert_eq!(drain.next(), Some(4));
    /// assert_eq!(drain.next(), Some(3));
    /// assert_eq!(drain.next(), Some(2));
    /// assert_eq!(drain.next(), None);
    /// drop(drain);
    ///
    /// assert_eq!(deque.len(), 2);
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn drain_back(&mut self, n: usize) -> Option<DrainBack<'_, T, N>> {
        DrainBack::new(self, n)
    }
}

/// An immutable iterator over an `ArrayDeque<T, N>`.
///
/// This struct is created by the [`iter`] method on [`ArrayDeque`].
///
/// [`iter`]: ArrayDeque::iter
pub struct Iter<'a, T, const N: usize>
where
    T: Default,
{
    inner: DequeIter<'a, ArrayDeque<T, N>, T>,
}

impl<'a, T, const N: usize> Iter<'a, T, N>
where
    T: Default,
{
    #[inline]
    fn new(deque: &'a ArrayDeque<T, N>) -> Iter<'a, T, N> {
        Iter {
            inner: DequeIter::new(deque),
        }
    }
}

impl<'a, T, const N: usize> Iterator for Iter<'a, T, N>
where
    T: Default,
{
    type Item = &'a T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'a, T, const N: usize> DoubleEndedIterator for Iter<'a, T, N>
where
    T: Default,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

/// A draining iterator which removes elements from the front of an
/// `ArrayDeque<T, N>`.
///
/// This struct is created by the [`drain_front`] method on [`ArrayDeque`].
///
/// [`drain_front`]: ArrayDeque::drain_front
pub struct DrainFront<'a, T, const N: usize>
where
    T: Default,
{
    inner: DequeDrain<'a, ArrayDeque<T, N>, T>,
}

impl<'a, T, const N: usize> DrainFront<'a, T, N>
where
    T: Default,
{
    #[inline]
    fn new(deque: &'a mut ArrayDeque<T, N>, n: usize) -> Option<DrainFront<'a, T, N>> {
        Some(DrainFront {
            inner: DequeDrain::front(deque, n)?,
        })
    }
}

impl<'a, T, const N: usize> Iterator for DrainFront<'a, T, N>
where
    T: Default,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

/// A draining iterator which removes elements from the back of an
/// `ArrayDeque<T, N>`.
///
/// This struct is created by the [`drain_back`] method on [`ArrayDeque`].
///
/// [`drain_back`]: ArrayDeque::drain_back
pub struct DrainBack<'a, T, const N: usize>
where
    T: Default,
{
    inner: DequeDrain<'a, ArrayDeque<T, N>, T>,
}

impl<'a, T, const N: usize> DrainBack<'a, T, N>
where
    T: Default,
{
    #[inline]
    fn new(deque: &'a mut ArrayDeque<T, N>, n: usize) -> Option<DrainBack<'a, T, N>> {
        Some(DrainBack {
            inner: DequeDrain::back(deque, n)?,
        })
    }
}

impl<'a, T, const N: usize> Iterator for DrainBack<'a, T, N>
where
    T: Default,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

#[cfg(feature = "serde")]
use core::{fmt, marker::PhantomData};

#[cfg(feature = "serde")]
use serde::{
    de::{Deserialize, Deserializer, Error, Expected, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};

#[cfg(feature = "serde")]
impl<T, const N: usize> serde::Serialize for ArrayDeque<T, N>
where
    T: Serialize + Default,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.len()))?;

        for element in self.iter() {
            seq.serialize_element(element)?;
        }

        seq.end()
    }
}

#[cfg(feature = "serde")]
#[doc(hidden)]
pub struct ExceededCapacity {
    capacity: usize,
}

#[cfg(feature = "serde")]
impl Expected for ExceededCapacity {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "a sequence of at most {} elements",
            self.capacity
        )
    }
}

#[cfg(feature = "serde")]
impl<'de, T, const N: usize> Deserialize<'de> for ArrayDeque<T, N>
where
    T: Deserialize<'de> + Default,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ArrayDequeVisitor<T, const N: usize> {
            phantom: core::marker::PhantomData<T>,
        }

        impl<'de, T, const N: usize> Visitor<'de> for ArrayDequeVisitor<T, N>
        where
            T: Deserialize<'de> + Default,
        {
            type Value = ArrayDeque<T, N>;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(formatter, "a sequence of at most {} elements", N)
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut deque = ArrayDeque::new();

                while let Some(elem) = seq.next_element()? {
                    deque.push_back(elem).map_err(|_| {
                        A::Error::invalid_length(deque.len() + 1, &ExceededCapacity { capacity: N })
                    })?;
                }

                Ok(deque)
            }
        }

        deserializer.deserialize_seq(ArrayDequeVisitor {
            phantom: PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::mem;

    extern crate alloc;
    use alloc::rc::Rc;

    #[test]
    fn empty_deque_has_zero_len() {
        let d0: ArrayDeque<(), 0> = ArrayDeque::new();
        assert_eq!(d0.len(), 0);

        let d1: ArrayDeque<(), 1> = ArrayDeque::new();
        assert_eq!(d1.len(), 0);

        let d3: ArrayDeque<(), 3> = ArrayDeque::new();
        assert_eq!(d3.len(), 0);
    }

    #[test]
    fn empty_deque_front_is_none() {
        let d0: ArrayDeque<(), 0> = ArrayDeque::new();
        assert_eq!(d0.front(), None);

        let d1: ArrayDeque<(), 1> = ArrayDeque::new();
        assert_eq!(d1.front(), None);

        let d3: ArrayDeque<(), 3> = ArrayDeque::new();
        assert_eq!(d3.front(), None);
    }

    #[test]
    fn empty_deque_back_is_none() {
        let d0: ArrayDeque<(), 0> = ArrayDeque::new();
        assert_eq!(d0.front(), None);

        let d1: ArrayDeque<(), 1> = ArrayDeque::new();
        assert_eq!(d1.front(), None);

        let d3: ArrayDeque<(), 3> = ArrayDeque::new();
        assert_eq!(d3.front(), None);
    }

    #[test]
    fn zero_capacity_is_both_empty_and_full() {
        let zero_cap: ArrayDeque<(), 0> = ArrayDeque::new();

        assert!(zero_cap.is_empty());
        assert!(zero_cap.is_full());
    }

    #[test]
    fn push_zero_capacity_is_error() {
        let mut zero_cap: ArrayDeque<(), 0> = ArrayDeque::new();

        assert!(zero_cap.push_front(()).is_err());
        assert!(zero_cap.push_back(()).is_err());
    }

    #[test]
    fn pop_zero_capacity_is_none() {
        let mut zero_cap: ArrayDeque<(), 0> = ArrayDeque::new();

        assert!(zero_cap.pop_front().is_none());
        assert!(zero_cap.pop_back().is_none());
    }

    #[test]
    fn push_full_linear_is_error() {
        let mut deque: ArrayDeque<(), 3> = ArrayDeque::new();

        deque.push_front(()).unwrap();
        deque.push_front(()).unwrap();
        deque.push_front(()).unwrap();

        assert!(deque.push_front(()).is_err());
        assert!(deque.push_back(()).is_err());
    }

    #[test]
    fn push_full_wrapped_is_error() {
        let mut deque: ArrayDeque<(), 3> = ArrayDeque::new();

        deque.push_front(()).unwrap();
        deque.push_front(()).unwrap();
        deque.push_back(()).unwrap();

        assert!(deque.push_front(()).is_err());
        assert!(deque.push_back(()).is_err());
    }

    #[test]
    fn pop_empty_is_none() {
        let mut deque: ArrayDeque<(), 3> = ArrayDeque::new();

        assert!(deque.pop_front().is_none());
        assert!(deque.pop_back().is_none());
    }

    #[test]
    fn push_front_one_becomes_front_and_back() {
        let mut deque: ArrayDeque<usize, 3> = ArrayDeque::new();

        deque.push_front(42).unwrap();
        assert_eq!(deque.front(), Some(&42));
        assert_eq!(deque.back(), Some(&42));
    }

    #[test]
    fn push_back_one_becomes_front_and_back() {
        let mut deque: ArrayDeque<usize, 3> = ArrayDeque::new();

        deque.push_back(42).unwrap();
        assert_eq!(deque.front(), Some(&42));
        assert_eq!(deque.back(), Some(&42));
    }

    #[test]
    fn push_both_ends_front_back() {
        let mut deque: ArrayDeque<&'static str, 3> = ArrayDeque::new();

        deque.push_back("back").unwrap();
        deque.push_front("front").unwrap();

        assert_eq!(deque.front(), Some(&"front"));
        assert_eq!(deque.back(), Some(&"back"));
    }

    #[test]
    fn push_pop_front() {
        let mut deque: ArrayDeque<&'static str, 3> = ArrayDeque::new();

        deque.push_front("front").unwrap();
        assert_eq!(deque.len(), 1);
        assert_eq!(deque.pop_front(), Some("front"));
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn push_pop_back() {
        let mut deque: ArrayDeque<&'static str, 3> = ArrayDeque::new();

        deque.push_back("back").unwrap();
        assert_eq!(deque.len(), 1);
        assert_eq!(deque.pop_back(), Some("back"));
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn push_front_then_back() {
        let mut deque: ArrayDeque<&'static str, 3> = ArrayDeque::new();

        deque.push_front("front").unwrap();
        assert_eq!(deque.len(), 1);
        deque.push_back("back").unwrap();
        assert_eq!(deque.len(), 2);

        let mut pop_front_front = deque.clone();
        let mut pop_front_back = deque.clone();
        let mut pop_back_front = deque.clone();
        let mut pop_back_back = deque.clone();

        assert_eq!(pop_front_front.pop_front(), Some("front"));
        assert_eq!(pop_front_front.pop_front(), Some("back"));

        assert_eq!(pop_front_back.pop_front(), Some("front"));
        assert_eq!(pop_front_back.pop_back(), Some("back"));

        assert_eq!(pop_back_front.pop_back(), Some("back"));
        assert_eq!(pop_back_front.pop_front(), Some("front"));

        assert_eq!(pop_back_back.pop_back(), Some("back"));
        assert_eq!(pop_back_back.pop_back(), Some("front"));
    }

    #[test]
    fn push_back_then_front() {
        let mut deque: ArrayDeque<&'static str, 3> = ArrayDeque::new();

        deque.push_back("back").unwrap();
        assert_eq!(deque.len(), 1);
        deque.push_front("front").unwrap();
        assert_eq!(deque.len(), 2);

        let mut pop_front_front = deque.clone();
        let mut pop_front_back = deque.clone();
        let mut pop_back_front = deque.clone();
        let mut pop_back_back = deque.clone();

        assert_eq!(pop_front_front.pop_front(), Some("front"));
        assert_eq!(pop_front_front.pop_front(), Some("back"));

        assert_eq!(pop_front_back.pop_front(), Some("front"));
        assert_eq!(pop_front_back.pop_back(), Some("back"));

        assert_eq!(pop_back_front.pop_back(), Some("back"));
        assert_eq!(pop_back_front.pop_front(), Some("front"));

        assert_eq!(pop_back_back.pop_back(), Some("back"));
        assert_eq!(pop_back_back.pop_back(), Some("front"));
    }

    #[test]
    fn iter_zero_capacity() {
        let deque: ArrayDeque<usize, 0> = ArrayDeque::new();
        let mut iter = deque.iter();

        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());
    }

    #[test]
    fn iter_forward() {
        let mut deque: ArrayDeque<usize, 5> = ArrayDeque::new();
        deque.push_back(0).unwrap();
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();
        deque.push_back(4).unwrap();

        let mut iter = deque.iter();
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_reverse() {
        let mut deque: ArrayDeque<usize, 5> = ArrayDeque::new();
        deque.push_back(4).unwrap();
        deque.push_back(3).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(1).unwrap();
        deque.push_back(0).unwrap();

        let mut iter = deque.iter().rev();
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next(), Some(&3));
        assert_eq!(iter.next(), Some(&4));
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn iter_alternate() {
        let mut deque: ArrayDeque<usize, 5> = ArrayDeque::new();
        deque.push_back(0).unwrap();
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();
        deque.push_back(4).unwrap();

        let mut iter = deque.iter();
        assert_eq!(iter.next(), Some(&0));
        assert_eq!(iter.next_back(), Some(&4));
        assert_eq!(iter.next(), Some(&1));
        assert_eq!(iter.next_back(), Some(&3));
        assert_eq!(iter.next(), Some(&2));
        assert_eq!(iter.next_back(), None);
        assert_eq!(iter.next(), None);
    }

    #[test]
    fn drain_zero_capacity() {
        let mut deque: ArrayDeque<(), 0> = ArrayDeque::new();
        assert!(deque.drain_front(1).is_none());
        assert!(deque.drain_back(1).is_none());
        assert!(deque.drain_front(0).unwrap().next().is_none());
        assert!(deque.drain_back(0).unwrap().next().is_none());
    }

    #[test]
    fn drain_runs_destructors_when_consumed() {
        let rc = Rc::new("refcount");

        let mut deque: ArrayDeque<Rc<&'static str>, 3> = ArrayDeque::new();
        deque.push_back(rc.clone()).unwrap();
        deque.push_back(rc.clone()).unwrap();
        deque.push_back(rc.clone()).unwrap();
        let drain = deque.drain_front(3).unwrap();
        drain.for_each(drop);

        assert_eq!(Rc::strong_count(&rc), 1);
    }

    #[test]
    fn drain_runs_destructors_when_dropped() {
        let rc = Rc::new("refcount");

        let mut deque: ArrayDeque<Rc<&'static str>, 3> = ArrayDeque::new();
        deque.push_back(rc.clone()).unwrap();
        deque.push_back(rc.clone()).unwrap();
        deque.push_back(rc.clone()).unwrap();
        let drain = deque.drain_front(3).unwrap();
        drop(drain);

        assert_eq!(Rc::strong_count(&rc), 1);
    }

    #[test]
    fn drain_removes_elements_when_leaked() {
        let mut deque: ArrayDeque<usize, 5> = ArrayDeque::new();
        deque.push_back(0).unwrap();
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();
        deque.push_back(4).unwrap();

        {
            let mut from_front = deque.clone();
            let drain = from_front.drain_front(3).unwrap();
            mem::forget(drain);
            assert_eq!(from_front.len(), 2);
            let mut iter = from_front.iter();
            assert_eq!(iter.next(), Some(&3));
            assert_eq!(iter.next(), Some(&4));
        }

        {
            let mut from_back = deque;
            let drain = from_back.drain_back(3).unwrap();
            mem::forget(drain);
            assert_eq!(from_back.len(), 2);
            let mut iter = from_back.iter();
            assert_eq!(iter.next(), Some(&0));
            assert_eq!(iter.next(), Some(&1));
        }
    }

    #[cfg(feature = "serde")]
    use serde_test::{assert_tokens, Token};

    #[cfg(feature = "serde")]
    #[test]
    fn serialize_deserialize() {
        let mut deque: ArrayDeque<String, 10> = ArrayDeque::new();
        deque.push_back("jumps".into()).unwrap();
        deque.push_front("fox".into()).unwrap();
        deque.push_back("over".into()).unwrap();
        deque.push_front("brown".into()).unwrap();
        deque.push_back("the".into()).unwrap();
        deque.push_front("quick".into()).unwrap();
        deque.push_back("lazy".into()).unwrap();
        deque.push_front("the".into()).unwrap();
        deque.push_back("dog".into()).unwrap();

        assert_tokens(
            &deque,
            &[
                Token::Seq { len: Some(9) },
                Token::Str("the".into()),
                Token::Str("quick".into()),
                Token::Str("brown".into()),
                Token::Str("fox".into()),
                Token::Str("jumps".into()),
                Token::Str("over".into()),
                Token::Str("the".into()),
                Token::Str("lazy".into()),
                Token::Str("dog".into()),
                Token::SeqEnd,
            ],
        );
    }
}