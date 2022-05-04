//! A double-ended queue with fixed capacity, backed by a slice.

use core::mem;

use crate::{
    meta::{Meta, MetaLayout},
    BaseDeque, CapacityError, DequeDrain, DequeIter,
};

#[cfg(feature = "serde")]
use serde::{
    de::{Deserialize, DeserializeSeed, Deserializer, Error, Expected, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};

#[derive(Clone, Debug)]
pub(crate) struct SliceMeta {
    capacity: usize,
    layout: MetaLayout,
}

impl SliceMeta {
    pub fn empty(capacity: usize) -> SliceMeta {
        SliceMeta {
            capacity,
            layout: MetaLayout::Empty,
        }
    }
}

impl Meta for SliceMeta {
    #[inline(always)]
    fn capacity(&self) -> usize {
        self.capacity
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

/// A double-ended queue with fixed capacity, backed by a slice.
///
/// The capacity of the deque is determined by the length of the slice.
#[derive(Debug)]
pub struct SliceDeque<'a, T>
where
    T: Default,
{
    meta: SliceMeta,
    items: &'a mut [T],
}

impl<'a, T> BaseDeque<T> for SliceDeque<'a, T>
where
    T: Default,
{
    type Meta = SliceMeta;

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
        self.items
    }

    #[inline(always)]
    fn items_mut(&mut self) -> &mut [T] {
        self.items
    }

    #[inline(always)]
    fn capacity(&self) -> usize {
        self.items.len()
    }
}

impl<'a, T> SliceDeque<'a, T>
where
    T: Default,
{
    /// Creates an empty `SliceDeque` backed by the provided slice.
    ///
    /// The elements in the slice are dropped and replaced with the default
    /// value of `T`.
    ///
    /// # Example
    /// ```
    /// # use holodeque::SliceDeque;
    /// # fn main() {
    /// let mut slice = ["these", "values", "will", "disappear"];
    /// let mut deque = SliceDeque::new_in(&mut slice);
    ///
    /// assert!(deque.is_empty());
    /// assert_eq!(deque.capacity(), 4);
    /// # }
    /// ```
    pub fn new_in(slice: &'a mut [T]) -> SliceDeque<'a, T> {
        let meta = SliceMeta::empty(slice.len());

        // Drop all existing values in the slice.
        for item in slice.iter_mut() {
            drop(mem::take(item));
        }

        SliceDeque { meta, items: slice }
    }

    /// Returns the maximum number of elements the deque may hold.
    ///
    /// This is the length of the backing slice.
    ///
    /// # Example
    /// ```
    /// # use holodeque::SliceDeque;
    /// # fn main() {
    /// let mut slice = [(), (), (), ()];
    /// let mut deque = SliceDeque::new_in(&mut slice);
    ///
    /// assert_eq!(deque.capacity(), 4);
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
    ///
    /// assert_eq!(deque.len(), 0);
    /// deque.push_back(1)?;
    /// deque.push_back(2)?;
    /// deque.push_back(3)?;
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
    ///
    /// deque.push_back(42)?;
    /// assert!(!deque.is_empty());
    /// deque.pop_front();
    /// assert!(deque.is_empty());
    /// # Ok(())
    /// # })().unwrap()
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
    /// # use holodeque::{SliceDeque, CapacityError};
    /// # fn main()  {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = ['\0', '\0', '\0'];
    /// let mut deque = SliceDeque::new_in(&mut slice);
    ///
    /// deque.push_back('a')?;
    /// deque.push_back('b')?;
    /// deque.push_back('c')?;
    ///
    /// assert_eq!(deque.front(), Some(&'a'));
    /// # Ok(())
    /// # })().unwrap()
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = ["", "", "", ""];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = ['\0', '\0', '\0'];
    /// let mut deque = SliceDeque::new_in(&mut slice);
    ///
    /// deque.push_back('a')?;
    /// deque.push_back('b')?;
    /// deque.push_back('c')?;
    ///
    /// assert_eq!(deque.back(), Some(&'c'));
    /// # Ok(())
    /// # })().unwrap()
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = ["", "", "", ""];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    /// `SliceDeque`.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0, 0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    /// of the `SliceDeque`.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0, 0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    /// # use holodeque::{SliceDeque, CapacityError};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
    ///
    /// deque.push_back(1)?;
    /// deque.push_back(2)?;
    /// deque.push_back(3)?;
    ///
    /// assert_eq!(deque.front(), Some(&1));
    /// assert_eq!(deque.back(), Some(&3));
    ///
    /// // Another element would exceed capacity, so this fails.
    /// let err = deque.push_back(4).unwrap_err();
    /// assert_eq!(err.into_inner(), 4);
    ///
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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

    /// Clears the `SliceDeque`, removing all values.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0, 0, 0, 0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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

    /// Shortens the `SliceDeque`, keeping the first `len` elements and dropping
    /// the rest.
    ///
    /// If `len` is greater than the `SliceDeque`'s current length, this has no
    /// effect.
    ///
    /// # Example
    ///
    /// ```
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0, 0, 0, 0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = ["", "", "", "", ""];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    pub fn iter(&self) -> Iter<'_, 'a, T> {
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    pub fn drain_front(&mut self, n: usize) -> Option<DrainFront<'_, 'a, T>> {
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
    /// # use holodeque::{CapacityError, SliceDeque};
    /// # fn main() {
    /// # (|| -> Result<(), CapacityError<_>> {
    /// let mut slice = [0, 0, 0, 0, 0];
    /// let mut deque = SliceDeque::new_in(&mut slice);
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
    pub fn drain_back(&mut self, n: usize) -> Option<DrainBack<'_, 'a, T>> {
        DrainBack::new(self, n)
    }
}

#[cfg(feature = "serde")]
impl<'a, 'de, T> SliceDeque<'a, T>
where
    T: Deserialize<'de> + Default,
{
    /// Extends the deque with the contents of a deserializer.
    pub fn extend_deserialize<D>(&mut self, deserializer: D) -> Result<(), D::Error>
    where
        D: Deserializer<'de>,
    {
        let extend = ExtendSliceDeque { deque: self };
        extend.deserialize(deserializer)?;
        Ok(())
    }
}

/// An immutable iterator over a `SliceDeque<'a, T>`.
///
/// This struct is created by the [`iter`] method on [`SliceDeque`].
///
/// [`iter`]: SliceDeque::iter
pub struct Iter<'it, 'a, T>
where
    T: Default,
{
    inner: DequeIter<'it, SliceDeque<'a, T>, T>,
}

impl<'it, 'a, T> Iter<'it, 'a, T>
where
    T: Default,
{
    #[inline]
    fn new(deque: &'it SliceDeque<'a, T>) -> Iter<'it, 'a, T> {
        Iter {
            inner: DequeIter::new(deque),
        }
    }
}

impl<'it, 'a, T> Iterator for Iter<'it, 'a, T>
where
    T: Default,
{
    type Item = &'it T;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next()
    }
}

impl<'it, 'a, T> DoubleEndedIterator for Iter<'it, 'a, T>
where
    T: Default,
{
    #[inline]
    fn next_back(&mut self) -> Option<Self::Item> {
        self.inner.next_back()
    }
}

/// A draining iterator which removes elements from the front of an
/// `SliceDeque<'a, T>`.
///
/// This struct is created by the [`drain_front`] method on [`SliceDeque`].
///
/// [`drain_front`]: SliceDeque::drain_front
pub struct DrainFront<'it, 'a, T>
where
    T: Default,
{
    inner: DequeDrain<'it, SliceDeque<'a, T>, T>,
}

impl<'it, 'a, T> DrainFront<'it, 'a, T>
where
    T: Default,
{
    #[inline]
    fn new(deque: &'it mut SliceDeque<'a, T>, n: usize) -> Option<DrainFront<'it, 'a, T>> {
        Some(DrainFront {
            inner: DequeDrain::front(deque, n)?,
        })
    }
}

impl<'it, 'a, T> Iterator for DrainFront<'it, 'a, T>
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
/// `SliceDeque<'a, T>`.
///
/// This struct is created by the [`drain_back`] method on [`SliceDeque`].
///
/// [`drain_back`]: SliceDeque::drain_back
pub struct DrainBack<'it, 'a, T>
where
    T: Default,
{
    inner: DequeDrain<'it, SliceDeque<'a, T>, T>,
}

impl<'it, 'a, T> DrainBack<'it, 'a, T>
where
    T: Default,
{
    #[inline]
    fn new(deque: &'it mut SliceDeque<'a, T>, n: usize) -> Option<DrainBack<'it, 'a, T>> {
        Some(DrainBack {
            inner: DequeDrain::back(deque, n)?,
        })
    }
}

impl<'it, 'a, T> Iterator for DrainBack<'it, 'a, T>
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
use core::fmt;

#[cfg(feature = "serde")]
impl<'a, T> serde::Serialize for SliceDeque<'a, T>
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
struct ExtendSliceDeque<'deque, 'slice, T>
where
    T: Default,
{
    deque: &'deque mut SliceDeque<'slice, T>,
}

#[cfg(feature = "serde")]
impl<'deque, 'slice, 'de, T> DeserializeSeed<'de> for ExtendSliceDeque<'deque, 'slice, T>
where
    T: Deserialize<'de> + Default,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ExtendSliceDequeVisitor<'deque, 'slice, T>
        where
            T: Default,
        {
            deque: &'deque mut SliceDeque<'slice, T>,
        }

        impl<'deque, 'slice, 'de, T> Visitor<'de> for ExtendSliceDequeVisitor<'deque, 'slice, T>
        where
            T: Deserialize<'de> + Default,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                write!(
                    formatter,
                    "a sequence of at most {} elements",
                    self.deque.capacity() - self.deque.len()
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                while let Some(elem) = seq.next_element()? {
                    self.deque.push_back(elem).map_err(|_| {
                        A::Error::invalid_length(
                            self.deque.len() + 1,
                            &ExceededCapacity {
                                capacity: self.deque.capacity(),
                            },
                        )
                    })?;
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(ExtendSliceDequeVisitor { deque: self.deque })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    extern crate alloc;
    use alloc::{rc::Rc, vec::Vec};

    #[test]
    fn empty_deque_has_zero_len() {
        let d0: SliceDeque<()> = SliceDeque::new_in(&mut []);
        assert_eq!(d0.len(), 0);

        let mut s1 = [()];
        let d1 = SliceDeque::new_in(&mut s1);
        assert_eq!(d1.len(), 0);

        let mut d3 = [(), (), ()];
        let d3 = SliceDeque::new_in(&mut d3);
        assert_eq!(d3.len(), 0);
    }

    #[test]
    fn empty_deque_front_is_none() {
        let d0: SliceDeque<()> = SliceDeque::new_in(&mut []);
        assert_eq!(d0.front(), None);

        let mut s1 = [()];
        let d1 = SliceDeque::new_in(&mut s1);
        assert_eq!(d1.front(), None);

        let mut s3 = [(), (), ()];
        let d3 = SliceDeque::new_in(&mut s3);
        assert_eq!(d3.front(), None);
    }

    #[test]
    fn empty_deque_back_is_none() {
        let d0: SliceDeque<()> = SliceDeque::new_in(&mut []);
        assert_eq!(d0.back(), None);

        let mut s1 = [()];
        let d1 = SliceDeque::new_in(&mut s1);
        assert_eq!(d1.back(), None);

        let mut s3 = [(), (), ()];
        let d3 = SliceDeque::new_in(&mut s3);
        assert_eq!(d3.back(), None);
    }

    #[test]
    fn zero_capacity_is_both_empty_and_full() {
        let zero_cap: SliceDeque<()> = SliceDeque::new_in(&mut []);

        assert!(zero_cap.is_empty());
        assert!(zero_cap.is_full());
    }

    #[test]
    fn push_zero_capacity_is_error() {
        let mut zero_cap = SliceDeque::new_in(&mut []);

        assert!(zero_cap.push_front(()).is_err());
        assert!(zero_cap.push_back(()).is_err());
    }

    #[test]
    fn pop_zero_capacity_is_none() {
        let mut zero_cap: SliceDeque<()> = SliceDeque::new_in(&mut []);

        assert_eq!(zero_cap.pop_front(), None);
        assert_eq!(zero_cap.pop_back(), None);
    }

    #[test]
    fn push_full_linear_is_error() {
        let mut slice = [(), (), ()];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_front(()).unwrap();
        deque.push_front(()).unwrap();
        deque.push_front(()).unwrap();

        assert!(deque.push_front(()).is_err());
        assert!(deque.push_back(()).is_err());
    }

    #[test]
    fn push_full_wrapped_is_error() {
        let mut slice = [(), (), ()];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_front(()).unwrap();
        deque.push_front(()).unwrap();
        deque.push_back(()).unwrap();

        assert!(deque.push_front(()).is_err());
        assert!(deque.push_back(()).is_err());
    }

    #[test]
    fn pop_empty_is_none() {
        let mut slice = [(), (), ()];
        let mut deque = SliceDeque::new_in(&mut slice);

        assert_eq!(deque.pop_front(), None);
        assert_eq!(deque.pop_back(), None);
    }

    #[test]
    fn push_front_one_becomes_front_and_back() {
        let mut slice = [0u32, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_front(42).unwrap();

        assert_eq!(deque.front(), Some(&42));
        assert_eq!(deque.back(), Some(&42));
    }

    #[test]
    fn push_back_one_becomes_front_and_back() {
        let mut slice = [0u32, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_back(42).unwrap();

        assert_eq!(deque.front(), Some(&42));
        assert_eq!(deque.back(), Some(&42));
    }

    #[test]
    fn push_front_becomes_wrapped() {
        let mut slice = [0u32, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_back(42).unwrap();
        deque.push_back(73).unwrap();
        deque.push_front(37).unwrap();

        assert_eq!(deque.front(), Some(&37));
        assert_eq!(deque.back(), Some(&73));
    }

    #[test]
    fn push_back_becomes_wrapped() {
        let mut slice = [0u32, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_front(42).unwrap();
        deque.push_front(73).unwrap();
        deque.push_back(37).unwrap();

        assert_eq!(deque.front(), Some(&73));
        assert_eq!(deque.back(), Some(&37));
    }

    #[test]
    fn push_both_ends_front_back() {
        let mut slice = ["", "", ""];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_back("back").unwrap();
        deque.push_front("front").unwrap();

        assert_eq!(deque.front(), Some(&"front"));
        assert_eq!(deque.back(), Some(&"back"));
    }

    #[test]
    fn push_pop_front() {
        let mut slice = ["", "", ""];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_front("front").unwrap();

        assert_eq!(deque.len(), 1);
        assert_eq!(deque.pop_front(), Some("front"));
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn push_pop_back() {
        let mut slice = ["", "", ""];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_back("back").unwrap();

        assert_eq!(deque.len(), 1);
        assert_eq!(deque.pop_back(), Some("back"));
        assert_eq!(deque.len(), 0);
    }

    #[test]
    fn push_front_then_back() {
        let mut slice_ff = ["", "", ""];
        let mut slice_fb = slice_ff.clone();
        let mut slice_bf = slice_ff.clone();
        let mut slice_bb = slice_ff.clone();

        let push_front_then_back = |deque: &mut SliceDeque<&'static str>| {
            deque.push_front("front").unwrap();
            assert_eq!(deque.len(), 1);
            deque.push_back("back").unwrap();
            assert_eq!(deque.len(), 2);
        };

        {
            let mut pop_front_front = SliceDeque::new_in(&mut slice_ff);
            push_front_then_back(&mut pop_front_front);

            assert_eq!(pop_front_front.pop_front(), Some("front"));
            assert_eq!(pop_front_front.pop_front(), Some("back"));
        }

        {
            let mut pop_front_back = SliceDeque::new_in(&mut slice_fb);
            push_front_then_back(&mut pop_front_back);

            assert_eq!(pop_front_back.pop_front(), Some("front"));
            assert_eq!(pop_front_back.pop_back(), Some("back"));
        }

        {
            let mut pop_back_front = SliceDeque::new_in(&mut slice_bf);
            push_front_then_back(&mut pop_back_front);

            assert_eq!(pop_back_front.pop_back(), Some("back"));
            assert_eq!(pop_back_front.pop_front(), Some("front"));
        }

        {
            let mut pop_back_back = SliceDeque::new_in(&mut slice_bb);
            push_front_then_back(&mut pop_back_back);

            assert_eq!(pop_back_back.pop_back(), Some("back"));
            assert_eq!(pop_back_back.pop_back(), Some("front"));
        }
    }

    #[test]
    fn push_back_then_front() {
        let mut slice_ff = ["", "", ""];
        let mut slice_fb = slice_ff.clone();
        let mut slice_bf = slice_ff.clone();
        let mut slice_bb = slice_ff.clone();

        let push_back_then_front = |deque: &mut SliceDeque<&'static str>| {
            deque.push_back("back").unwrap();
            assert_eq!(deque.len(), 1);
            deque.push_front("front").unwrap();
            assert_eq!(deque.len(), 2);
        };

        {
            let mut pop_front_front = SliceDeque::new_in(&mut slice_ff);
            push_back_then_front(&mut pop_front_front);

            assert_eq!(pop_front_front.pop_front(), Some("front"));
            assert_eq!(pop_front_front.pop_front(), Some("back"));
        }

        {
            let mut pop_front_back = SliceDeque::new_in(&mut slice_fb);
            push_back_then_front(&mut pop_front_back);

            assert_eq!(pop_front_back.pop_front(), Some("front"));
            assert_eq!(pop_front_back.pop_back(), Some("back"));
        }

        {
            let mut pop_back_front = SliceDeque::new_in(&mut slice_bf);
            push_back_then_front(&mut pop_back_front);

            assert_eq!(pop_back_front.pop_back(), Some("back"));
            assert_eq!(pop_back_front.pop_front(), Some("front"));
        }

        {
            let mut pop_back_back = SliceDeque::new_in(&mut slice_bb);
            push_back_then_front(&mut pop_back_back);

            assert_eq!(pop_back_back.pop_back(), Some("back"));
            assert_eq!(pop_back_back.pop_back(), Some("front"));
        }
    }

    #[test]
    fn clear_makes_empty() {
        let mut slice = [0, 0, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);

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
        let mut slice = [0, 0, 0, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_back(42).unwrap();
        assert_eq!(deque.len(), 1);
        deque.truncate(5);
        assert_eq!(deque.len(), 1);
    }

    #[test]
    fn truncate_longer_reduces_len() {
        let mut slice = [0, 0, 0, 0, 0, 0, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);

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
        let deque: SliceDeque<()> = SliceDeque::new_in(&mut []);
        let mut iter = deque.iter();

        assert!(iter.next().is_none());
        assert!(iter.next_back().is_none());
    }

    #[test]
    fn iter_forward() {
        let mut slice = [0, 0, 0, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);
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
        let mut slice = [0, 0, 0, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);
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
        let mut slice = [0, 0, 0, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);
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
        let mut slice = [0, 0, 0, 0, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);

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
        let mut slice = [0, 0, 0, 0, 0, 0];
        let mut deque = SliceDeque::new_in(&mut slice);

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
        let mut deque: SliceDeque<()> = SliceDeque::new_in(&mut []);
        assert!(deque.drain_front(1).is_none());
        assert!(deque.drain_back(1).is_none());
        assert!(deque.drain_front(0).unwrap().next().is_none());
        assert!(deque.drain_back(0).unwrap().next().is_none());
    }

    #[test]
    fn drain_runs_destructors_when_consumed() {
        let rc = Rc::new("refcount");

        let mut slice = [Rc::new(""), Rc::new(""), Rc::new("")];
        let mut deque = SliceDeque::new_in(&mut slice);
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

        let mut slice = [Rc::new(""), Rc::new(""), Rc::new("")];
        let mut deque = SliceDeque::new_in(&mut slice);
        deque.push_back(rc.clone()).unwrap();
        deque.push_back(rc.clone()).unwrap();
        deque.push_back(rc.clone()).unwrap();
        let drain = deque.drain_front(3).unwrap();
        drop(drain);

        assert_eq!(Rc::strong_count(&rc), 1);
    }

    #[test]
    fn drain_removes_elements_when_leaked() {
        let populate = |deque: &mut SliceDeque<_>| {
            deque.push_back(0).unwrap();
            deque.push_back(1).unwrap();
            deque.push_back(2).unwrap();
            deque.push_back(3).unwrap();
            deque.push_back(4).unwrap();
        };

        {
            let mut slice = [0, 0, 0, 0, 0];
            let mut from_front = SliceDeque::new_in(&mut slice);
            populate(&mut from_front);

            let drain = from_front.drain_front(3).unwrap();
            mem::forget(drain);
            assert_eq!(from_front.len(), 2);
            let mut iter = from_front.iter();
            assert_eq!(iter.next(), Some(&3));
            assert_eq!(iter.next(), Some(&4));
        }

        {
            let mut slice = [0, 0, 0, 0, 0];
            let mut from_back = SliceDeque::new_in(&mut slice);
            populate(&mut from_back);

            let drain = from_back.drain_back(3).unwrap();
            mem::forget(drain);
            assert_eq!(from_back.len(), 2);
            let mut iter = from_back.iter();
            assert_eq!(iter.next(), Some(&0));
            assert_eq!(iter.next(), Some(&1));
        }
    }

    #[cfg(feature = "serde")]
    use serde_test::{assert_ser_tokens, Token};

    #[cfg(feature = "serde")]
    #[test]
    fn serialize() {
        let mut slice = ["", "", "", "", "", "", "", "", "", ""];
        let mut deque = SliceDeque::new_in(&mut slice);

        deque.push_back("jumps".into()).unwrap();
        deque.push_front("fox".into()).unwrap();
        deque.push_back("over".into()).unwrap();
        deque.push_front("brown".into()).unwrap();
        deque.push_back("the".into()).unwrap();
        deque.push_front("quick".into()).unwrap();
        deque.push_back("lazy".into()).unwrap();
        deque.push_front("the".into()).unwrap();
        deque.push_back("dog".into()).unwrap();

        assert_ser_tokens(
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
