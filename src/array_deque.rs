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

    /// Returns a pair of slices which contain, in order, the elements of the
    /// `ArrayDeque`.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<u32, 6> = ArrayDeque::new();
    ///
    /// deque.push_front(3)?;
    /// deque.push_front(6)?;
    /// deque.push_front(9)?;
    /// deque.push_back(5)?;
    /// deque.push_back(10)?;
    /// deque.push_back(15)?;
    ///
    /// let (first, second) = deque.as_slices();
    /// assert_eq!(first, &[9, 6, 3]);
    /// assert_eq!(second, &[5, 10, 15]);
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn as_slices(&self) -> (&[T], &[T]) {
        BaseDeque::as_slices(self)
    }

    /// Returns a pair of mutable slices which contain, in order, the elements
    /// of the `ArrayDeque`.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<u32, 6> = ArrayDeque::new();
    ///
    /// deque.push_front(3)?;
    /// deque.push_front(6)?;
    /// deque.push_front(9)?;
    /// deque.push_back(5)?;
    /// deque.push_back(10)?;
    /// deque.push_back(15)?;
    ///
    /// let (first_mut, second_mut) = deque.as_mut_slices();
    /// for item in first_mut {
    ///     *item -= 1;
    /// }
    /// for item in second_mut {
    ///     *item += 1;
    /// }
    ///
    /// let (first, second) = deque.as_slices();
    /// assert_eq!(first, &[8, 5, 2]);
    /// assert_eq!(second, &[6, 11, 16]);
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn as_mut_slices(&mut self) -> (&mut [T], &mut [T]) {
        BaseDeque::as_mut_slices(self)
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

    /// Clears the `ArrayDeque`, removing all values.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<usize, 8> = ArrayDeque::new();
    ///
    /// for i in 0..deque.capacity() / 2 {
    ///     deque.push_front(i)?;
    ///     deque.push_back(i)?;
    /// }
    ///
    /// assert_eq!(deque.len(), 8);
    /// deque.clear();
    /// assert!(deque.is_empty());
    ///
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn clear(&mut self) {
        BaseDeque::clear(self)
    }

    /// Shortens the `ArrayDeque`, keeping the first `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater than the `ArrayDeque`'s current length, this has no
    /// effect.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{ArrayDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut deque: ArrayDeque<usize, 8> = ArrayDeque::new();
    ///
    /// deque.push_back(5)?;
    /// deque.push_back(10)?;
    /// deque.push_back(15)?;
    /// deque.push_back(20)?;
    /// deque.push_back(25)?;
    ///
    /// assert_eq!(deque.len(), 5);
    /// deque.truncate(2);
    /// assert_eq!(deque.len(), 2);
    ///
    /// # Ok(())
    /// # })().unwrap();
    /// # }
    /// ```
    #[inline]
    pub fn truncate(&mut self, len: usize) {
        BaseDeque::truncate(self, len)
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

#[cfg(all(feature = "std", test))]
impl<T, const N: usize> quickcheck::Arbitrary for ArrayDeque<T, N>
where
    T: quickcheck::Arbitrary + std::fmt::Debug + Default,
{
    fn arbitrary(g: &mut quickcheck::Gen) -> Self {
        use crate::DequeEnd;

        let mut deque = ArrayDeque::new();
        let len = usize::arbitrary(g) % N;

        for _ in 0..len {
            let val = T::arbitrary(g);
            match g.choose(&[DequeEnd::Front, DequeEnd::Back]).unwrap() {
                DequeEnd::Front => deque.push_front(val).unwrap(),
                DequeEnd::Back => deque.push_back(val).unwrap(),
            }
        }

        deque
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        if self.is_empty() {
            Box::new(std::iter::empty())
        } else {
            let mut less_front = self.clone();
            less_front.pop_front();

            let mut less_back = self.clone();
            less_back.pop_back();

            Box::new(vec![less_front, less_back].into_iter())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::mem;

    extern crate alloc;
    use alloc::{rc::Rc, vec::Vec};

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
    fn clear_makes_empty() {
        let mut deque: ArrayDeque<u32, 4> = ArrayDeque::new();

        deque.push_back(0).unwrap();
        deque.push_back(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();

        assert_eq!(deque.len(), 4);
        deque.clear();
        assert!(deque.is_empty());

        deque.push_front(0).unwrap();
        deque.push_front(1).unwrap();
        deque.push_front(2).unwrap();
        deque.push_front(3).unwrap();

        assert_eq!(deque.len(), 4);
        deque.clear();
        assert!(deque.is_empty());

        deque.push_back(0).unwrap();
        deque.push_back(1).unwrap();
        deque.push_front(2).unwrap();
        deque.push_front(3).unwrap();

        assert_eq!(deque.len(), 4);
        deque.clear();
        assert!(deque.is_empty());

        deque.push_front(0).unwrap();
        deque.push_front(1).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(3).unwrap();

        assert_eq!(deque.len(), 4);
        deque.clear();
        assert!(deque.is_empty());
    }

    #[test]
    fn truncate_shorter_has_no_effect() {
        let mut deque: ArrayDeque<u32, 5> = ArrayDeque::new();

        deque.push_back(42).unwrap();
        assert_eq!(deque.len(), 1);
        deque.truncate(5);
        assert_eq!(deque.len(), 1);
    }

    #[test]
    fn truncate_longer_reduces_len() {
        let mut deque: ArrayDeque<u32, 8> = ArrayDeque::new();

        deque.push_back(5).unwrap();
        deque.push_back(10).unwrap();
        deque.push_back(15).unwrap();
        deque.push_back(20).unwrap();
        deque.push_back(25).unwrap();
        deque.push_back(30).unwrap();
        deque.push_back(35).unwrap();

        assert_eq!(deque.len(), 7);
        deque.truncate(4);
        assert_eq!(deque.len(), 4);
        assert_eq!(deque.front(), Some(&5));
        assert_eq!(deque.back(), Some(&20));
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
    fn iter_has_same_order_as_slices() {
        let mut deque: ArrayDeque<u32, 6> = ArrayDeque::new();

        deque.push_front(3).unwrap();
        deque.push_front(5).unwrap();
        deque.push_front(7).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(4).unwrap();
        deque.push_back(6).unwrap();

        let from_slices = {
            let mut v = Vec::new();

            let (first, second) = deque.as_slices();
            for &item in first.iter().chain(second.iter()) {
                v.push(item);
            }

            v
        };

        let from_iter = deque.iter().copied().collect::<Vec<_>>();

        assert_eq!(from_slices, from_iter);
    }

    #[test]
    fn slices_and_mut_slices_are_eq() {
        let mut deque: ArrayDeque<u32, 6> = ArrayDeque::new();

        deque.push_front(3).unwrap();
        deque.push_front(5).unwrap();
        deque.push_front(7).unwrap();
        deque.push_back(2).unwrap();
        deque.push_back(4).unwrap();
        deque.push_back(6).unwrap();

        let (s1, s2) = deque.as_slices();
        let v1 = Vec::from(s1);
        let v2 = Vec::from(s2);

        let (m1, m2) = deque.as_mut_slices();
        assert_eq!(v1, m1);
        assert_eq!(v2, m2);
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

    #[cfg(feature = "std")]
    quickcheck::quickcheck! {
        fn qc_front_unchanged_when_back_popped(deque: ArrayDeque<u8, 128>) -> bool {
            if deque.len() <= 1 {
                // pop_back() would remove front.
                return true;
            }

            let mut cloned = deque.clone();
            cloned.pop_back().unwrap();

            deque.front() == cloned.front()
        }

        fn qc_back_unchanged_when_front_popped(deque: ArrayDeque<u8, 128>) -> bool {
            if deque.len() <= 1 {
                // pop_front() would remove back.
                return true;
            }

            let mut cloned = deque.clone();
            cloned.pop_front().unwrap();

            deque.back() == cloned.back()
        }

        fn qc_truncate_produces_correct_len(deque: ArrayDeque<u8, 128>, len: usize) -> bool {
            let len = len % deque.capacity();
            let longer = deque.len() > len;

            let mut deque = deque;
            deque.truncate(len);

            if longer {
                deque.len() == len
            } else {
                deque.len() <= len
            }
        }

        fn qc_iter_produces_len_elements(deque: ArrayDeque<u8, 128>) -> bool {
            let mut count = 0;

            for _ in deque.iter() {
                count += 1;
            }

            count == deque.len()
        }

        fn qc_drain_front_produces_n_elements(deque: ArrayDeque<u8, 128>, n: usize) -> bool {
            let n = n % deque.len().max(1);
            let mut count = 0;
            let mut deque = deque;

            for _ in deque.drain_front(n).unwrap() {
                count += 1;
            }

            count == n
        }

        fn qc_drain_back_produces_n_elements(deque: ArrayDeque<u8, 128>, n: usize) -> bool {
            let n = n % deque.len().max(1);
            let mut count = 0;
            let mut deque = deque;

            for _ in deque.drain_back(n).unwrap() {
                count += 1;
            }

            count == n
        }
    }
}
