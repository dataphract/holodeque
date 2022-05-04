use core::{num::NonZeroUsize, ops::Range};

use crate::DequeEnd;

/// Metadata tracking the layout of the deque's backing array.
#[derive(Copy, Clone, Debug)]
pub enum MetaLayout {
    /// The deque is empty.
    Empty,

    /// The first item of the deque occurs before the last item of the deque in
    /// the backing array.
    Linear {
        /// The index of the first item of the deque.
        first: usize,

        /// The number of items in the deque.
        len: NonZeroUsize,
    },

    /// The first item of the deque occurs after the last item of the deque in
    /// the backing array.
    Wrapped {
        /// The length of the wrapped portion of the deque.
        ///
        /// The wrapped portion begins at index 0 in the backing array.
        wrap_len: NonZeroUsize,

        /// The length of the unused portion of the backing array.
        gap_len: usize,
    },
}

/// A trait for deque layout metadata.
pub trait Meta: Clone + Sized {
    /// Returns the capacity of the deque.
    fn capacity(&self) -> usize;

    /// Returns the layout of the deque's backing store.
    fn layout(&self) -> MetaLayout;

    /// Sets the layout of the deque's backing store.
    fn set_layout(&mut self, layout: MetaLayout);

    /// Returns the number of elements in the deque.
    fn len(&self) -> usize {
        match self.layout() {
            MetaLayout::Empty => 0,
            MetaLayout::Linear { len, .. } => {
                debug_assert!(len.get() <= self.capacity());
                len.get()
            }
            MetaLayout::Wrapped { gap_len, .. } => {
                let len = self.capacity() - gap_len;
                debug_assert!(len <= self.capacity());
                len
            }
        }
    }

    fn as_ranges(&self) -> (Range<usize>, Range<usize>) {
        match self.layout() {
            MetaLayout::Empty => (0..0, 0..0),
            MetaLayout::Linear { first, len } => (first..first + len.get(), 0..0),
            MetaLayout::Wrapped { wrap_len, gap_len } => {
                let start = wrap_len.get() + gap_len;
                (start..self.len(), 0..wrap_len.get())
            }
        }
    }

    /// Removes all indices from the deque, returning an iterator over the
    /// removed indices.
    fn clear(&mut self) -> MetaDrain<Self> {
        let drain = MetaDrain {
            meta: self.clone(),
            remaining: self.len(),
            end: DequeEnd::Front,
        };

        self.set_layout(MetaLayout::Empty);

        drain
    }

    /// Returns the index of the first element of the deque.
    ///
    /// If the deque is empty, `None` is returned.
    fn front(&self) -> Option<usize> {
        match self.layout() {
            MetaLayout::Empty => None,
            MetaLayout::Linear { first, .. } => {
                debug_assert!(first < self.capacity());
                Some(first)
            }
            MetaLayout::Wrapped { wrap_len, gap_len } => {
                let first = wrap_len.get() + gap_len;
                debug_assert!(first < self.capacity());
                Some(first)
            }
        }
    }

    /// Returns the index of the last element of the deque.
    ///
    /// If the deque is empty, `None` is returned.
    fn back(&self) -> Option<usize> {
        match self.layout() {
            MetaLayout::Empty => None,
            MetaLayout::Linear { first, len } => {
                let last = first + len.get() - 1;
                debug_assert!(last < self.capacity());
                Some(last)
            }
            MetaLayout::Wrapped { wrap_len, .. } => {
                let last = wrap_len.get() - 1;
                debug_assert!(last < self.capacity());
                Some(last)
            }
        }
    }

    /// Reserves an index at the front of the deque.
    fn reserve_front(&mut self) -> Option<usize> {
        if self.capacity() == 0 {
            return None;
        }

        match self.layout() {
            MetaLayout::Empty => {
                self.set_layout(MetaLayout::Linear {
                    first: self.capacity() - 1,
                    len: NonZeroUsize::new(1).unwrap(),
                });

                Some(self.capacity() - 1)
            }

            MetaLayout::Linear { len, .. } if len.get() == self.capacity() => None,

            MetaLayout::Linear { first: 0, len } => {
                self.set_layout(MetaLayout::Wrapped {
                    wrap_len: len,
                    gap_len: self.capacity() - (len.get() + 1),
                });

                Some(self.capacity() - 1)
            }

            MetaLayout::Linear { first, len } => {
                let new_first = first - 1;

                self.set_layout(MetaLayout::Linear {
                    first: new_first,
                    len: NonZeroUsize::new(len.get() + 1).unwrap(),
                });

                Some(new_first)
            }

            // If gap has zero len, the deque is full.
            MetaLayout::Wrapped { gap_len: 0, .. } => None,

            MetaLayout::Wrapped { wrap_len, gap_len } => {
                let new_gap_len = gap_len - 1;

                self.set_layout(MetaLayout::Wrapped {
                    wrap_len,
                    gap_len: new_gap_len,
                });

                Some(wrap_len.get() + new_gap_len)
            }
        }
    }

    /// Reserves an index at the back of the deque.
    fn reserve_back(&mut self) -> Option<usize> {
        if self.capacity() == 0 {
            return None;
        }

        match self.layout() {
            MetaLayout::Empty => {
                self.set_layout(MetaLayout::Linear {
                    first: 0,
                    len: NonZeroUsize::new(1).unwrap(),
                });

                Some(0)
            }

            MetaLayout::Linear { len, .. } if len.get() == self.capacity() => None,

            MetaLayout::Linear { first, len } if first + len.get() == self.capacity() => {
                self.set_layout(MetaLayout::Wrapped {
                    wrap_len: NonZeroUsize::new(1).unwrap(),
                    gap_len: self.capacity() - (len.get() + 1),
                });

                Some(0)
            }

            MetaLayout::Linear { first, len } => {
                let reserved = first + len.get();

                self.set_layout(MetaLayout::Linear {
                    first,
                    len: NonZeroUsize::new(len.get() + 1).unwrap(),
                });

                Some(reserved)
            }

            MetaLayout::Wrapped { gap_len: 0, .. } => None,

            MetaLayout::Wrapped { wrap_len, gap_len } => {
                let reserved = wrap_len.get();

                self.set_layout(MetaLayout::Wrapped {
                    wrap_len: NonZeroUsize::new(wrap_len.get() + 1).unwrap(),
                    gap_len: gap_len - 1,
                });

                Some(reserved)
            }
        }
    }

    /// Frees an index at the front of the deque.
    fn free_front(&mut self) -> Option<usize> {
        if self.capacity() == 0 {
            return None;
        }

        match self.layout() {
            MetaLayout::Empty => None,

            MetaLayout::Linear { first, len } => {
                let freed = first;

                let new_layout = match NonZeroUsize::new(len.get() - 1) {
                    Some(new_len) => MetaLayout::Linear {
                        first: first + 1,
                        len: new_len,
                    },

                    None => MetaLayout::Empty,
                };

                self.set_layout(new_layout);

                Some(freed)
            }

            MetaLayout::Wrapped { wrap_len, gap_len } => {
                let freed = wrap_len.get() + gap_len;

                let new_layout = if freed == self.capacity() - 1 {
                    MetaLayout::Linear {
                        first: 0,
                        len: wrap_len,
                    }
                } else {
                    MetaLayout::Wrapped {
                        wrap_len,
                        gap_len: gap_len + 1,
                    }
                };

                self.set_layout(new_layout);

                Some(freed)
            }
        }
    }

    /// Drains `n` indices from the front of the deque.
    fn drain_front(&mut self, n: usize) -> Option<MetaDrain<Self>> {
        // This checks that n <= len.
        let drain = MetaDrain::front(self.clone(), n)?;

        match self.layout() {
            // n must be zero, so this is a no-op.
            MetaLayout::Empty => (),

            MetaLayout::Linear { first, len } => match NonZeroUsize::new(len.get() - n) {
                Some(new_len) => {
                    self.set_layout(MetaLayout::Linear {
                        first: first + n,
                        len: new_len,
                    });
                }

                None => {
                    self.set_layout(MetaLayout::Empty);
                }
            },

            MetaLayout::Wrapped { wrap_len, gap_len } => {
                let front_len = self.capacity() - (wrap_len.get() + gap_len);

                if n >= front_len {
                    let first = n - front_len;

                    match NonZeroUsize::new(wrap_len.get() - first) {
                        Some(new_len) => {
                            self.set_layout(MetaLayout::Linear {
                                first,
                                len: new_len,
                            });
                        }

                        None => self.set_layout(MetaLayout::Empty),
                    }
                } else {
                    self.set_layout(MetaLayout::Wrapped {
                        wrap_len,
                        gap_len: gap_len + n,
                    });
                }
            }
        }

        Some(drain)
    }

    /// Frees an index at the back of the deque.
    fn free_back(&mut self) -> Option<usize> {
        if self.capacity() == 0 {
            return None;
        }

        match self.layout() {
            MetaLayout::Empty => None,

            MetaLayout::Linear { first, len } => {
                let freed = first + len.get() - 1;

                let new_layout = match NonZeroUsize::new(len.get() - 1) {
                    Some(new_len) => MetaLayout::Linear {
                        first,
                        len: new_len,
                    },
                    None => MetaLayout::Empty,
                };

                self.set_layout(new_layout);

                Some(freed)
            }

            MetaLayout::Wrapped { wrap_len, gap_len } => {
                let (freed, new_layout) = match NonZeroUsize::new(wrap_len.get() - 1) {
                    Some(new_wrap_len) => (
                        new_wrap_len.get(),
                        MetaLayout::Wrapped {
                            wrap_len: new_wrap_len,
                            gap_len: gap_len + 1,
                        },
                    ),

                    None => (
                        0,
                        MetaLayout::Linear {
                            first: gap_len + 1,
                            len: NonZeroUsize::new(self.capacity() - (gap_len + 1)).unwrap(),
                        },
                    ),
                };

                self.set_layout(new_layout);

                Some(freed)
            }
        }
    }

    /// Drains `n` indices from the back of the deque.
    fn drain_back(&mut self, n: usize) -> Option<MetaDrain<Self>> {
        let drain = MetaDrain::back(self.clone(), n)?;

        match self.layout() {
            MetaLayout::Empty => (),

            MetaLayout::Linear { first, len } => match NonZeroUsize::new(len.get() - n) {
                Some(new_len) => {
                    self.set_layout(MetaLayout::Linear {
                        first,
                        len: new_len,
                    });
                }

                None => self.set_layout(MetaLayout::Empty),
            },

            MetaLayout::Wrapped { wrap_len, gap_len } => {
                if n >= wrap_len.get() {
                    let total_len = self.capacity() - gap_len;

                    let new_layout = match NonZeroUsize::new(total_len - n) {
                        Some(new_len) => MetaLayout::Linear {
                            first: wrap_len.get() + gap_len,
                            len: new_len,
                        },
                        None => MetaLayout::Empty,
                    };

                    self.set_layout(new_layout);
                } else {
                    self.set_layout(MetaLayout::Wrapped {
                        wrap_len: NonZeroUsize::new(wrap_len.get() - n).unwrap(),
                        gap_len: gap_len + n,
                    });
                }
            }
        }

        Some(drain)
    }
}

pub struct MetaDrain<M>
where
    M: Meta,
{
    meta: M,
    remaining: usize,
    end: DequeEnd,
}

impl<M> MetaDrain<M>
where
    M: Meta,
{
    /// Creates an iterator that drains `n` indices from the front of the deque.
    ///
    /// If `n` exceeds the number of items in the deque, `None` is returned.
    pub fn front(meta: M, n: usize) -> Option<MetaDrain<M>> {
        if n > meta.len() {
            None
        } else {
            Some(MetaDrain {
                meta,
                remaining: n,
                end: DequeEnd::Front,
            })
        }
    }

    /// Creates an iterator that drains `n` indices from the back of the deque.
    ///
    /// If `n` exceeds the number of items in the deque, `None` is returned.
    pub fn back(meta: M, n: usize) -> Option<MetaDrain<M>> {
        if n > meta.len() {
            None
        } else {
            Some(MetaDrain {
                meta,
                remaining: n,
                end: DequeEnd::Back,
            })
        }
    }
}

impl<M> Iterator for MetaDrain<M>
where
    M: Meta,
{
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining > 0 {
            self.remaining -= 1;

            let index = match self.end {
                DequeEnd::Front => self.meta.free_front().unwrap(),
                DequeEnd::Back => self.meta.free_back().unwrap(),
            };

            Some(index)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}
