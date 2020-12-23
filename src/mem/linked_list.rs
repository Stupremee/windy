//! Intrusive linked list implementation.

use core::{
    marker::PhantomData,
    ptr::{self, NonNull},
};

/// Intrusive linked lsit used in the buddy allocator.
#[derive(Clone, Copy)]
pub struct LinkedList {
    head: *mut usize,
}

impl LinkedList {
    /// Create a new `LinkedList`.
    pub const fn new() -> Self {
        Self {
            head: ptr::null_mut(),
        }
    }

    /// Returns whether this list is emtpy.
    pub fn is_empty(&self) -> bool {
        self.head.is_null()
    }

    /// Push the given item to the front of the list.
    ///
    /// Takes the pointer into your data structure, that contains the
    /// address for the next element.
    ///
    /// # Safety
    ///
    /// `item` pointer must be valid to write and not-null.
    /// See [`core::ptr`] for more safety information.
    pub unsafe fn push(&mut self, item: *mut usize) {
        *item = self.head as usize;
        self.head = item;
    }

    /// Removes the first item from this list.
    pub fn pop(&mut self) -> Option<NonNull<usize>> {
        if self.is_empty() {
            return None;
        }

        // SAFETY
        // We checked above if `head` is null.
        let item = unsafe { NonNull::new_unchecked(self.head) };
        self.head = unsafe { *item.as_ptr() as *mut usize };
        Some(item)
    }

    /// Returns an immutable iterator over the elements of `self`.
    pub fn iter(&self) -> Iter<'_> {
        Iter {
            head: self.head,
            _lifetime: PhantomData,
        }
    }

    /// Returns a mutable iterator over the nodes of
    /// this linked list.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        IterMut {
            prev: &mut self.head as *mut *mut usize as *mut usize,
            head: self.head,
            _lifetime: PhantomData,
        }
    }
}

pub struct Iter<'list> {
    head: *mut usize,
    _lifetime: PhantomData<&'list LinkedList>,
}

impl Iterator for Iter<'_> {
    type Item = NonNull<usize>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head.is_null() {
            None
        } else {
            // SAFETY
            // We checked above is `self.head` is NULL, the rest
            // must be guaranteed by the caller of `push`.
            let item = unsafe { NonNull::new_unchecked(self.head) };
            self.head = unsafe { *item.as_ptr() as *mut _ };
            Some(item)
        }
    }
}

/// Represents a mutable node of [`LinkedList`].
pub struct ListNode {
    prev: *mut usize,
    head: *mut usize,
}

impl ListNode {
    /// Remove this node from the list.
    pub fn pop(self) -> *mut usize {
        // SAFETY
        // Safety arguments must be provided by the caller of `push`.
        unsafe {
            *self.prev = *self.head;
        }
        self.head
    }

    /// Returns the pointer to the address
    /// of the next node.
    pub fn addr(&self) -> *mut usize {
        self.head
    }
}

pub struct IterMut<'list> {
    head: *mut usize,
    prev: *mut usize,
    _lifetime: PhantomData<&'list mut LinkedList>,
}

impl Iterator for IterMut<'_> {
    type Item = ListNode;

    fn next(&mut self) -> Option<Self::Item> {
        if self.head.is_null() {
            None
        } else {
            let node = ListNode {
                head: self.head,
                prev: self.prev,
            };
            self.prev = self.head;
            // SAFETY
            // Safety arguments must be provided by the caller of `push`.
            self.head = unsafe { *self.head as *mut usize };
            Some(node)
        }
    }
}
