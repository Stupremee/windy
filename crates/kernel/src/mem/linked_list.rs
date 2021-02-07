//! Implementation of a intrusive linked list.

use core::{marker::PhantomData, ptr::NonNull};

/// Intrusive linked list.
pub struct LinkedList {
    head: Option<NonNull<usize>>,
}

impl LinkedList {
    /// Creates a new, empty linked list.
    pub const fn new() -> Self {
        Self { head: None }
    }

    /// Check if this list is empty
    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    /// Push the given item to the front of the list.
    ///
    /// Takes a pointer that points into your data structure, where the pointer
    /// for the next element lives.
    ///
    /// # Safety
    ///
    /// `item` pointer must be valid for writes and must not outlive this list.
    pub unsafe fn push(&mut self, item: NonNull<usize>) {
        *item.as_ptr() = self.head.map(|ptr| ptr.as_ptr() as usize).unwrap_or(0);
        self.head = Some(item);
    }

    /// Pop one element from the head of this list.
    pub fn pop(&mut self) -> Option<NonNull<usize>> {
        if let Some(head) = self.head {
            // SAFETY
            // `head` is not null, and the other guarantees must be guaranteed by `push`.
            self.head = NonNull::new(unsafe { *head.as_ptr() as *mut usize });
            Some(head)
        } else {
            None
        }
    }

    /// Return an iterator over the mutable [`ListNode`]s of this list.
    pub fn iter_mut(&mut self) -> IterMut<'_> {
        // this is where things get weird
        //
        // we create a mutable pointer to the pointer `self.head`, so we end up with
        // `*mut *mut usize`. we then cast that to `*mut usize`, so we get back a pointer, that
        // points to the address of the pointer inside `self.head`.
        let prev = self
            .head
            .as_mut()
            .map(|head| head.as_ptr() as *mut *mut usize)
            .map(|head| head as *mut usize)
            .and_then(NonNull::new);

        IterMut {
            prev,
            head: self.head,
            _lifetime: PhantomData,
        }
    }
}

/// Represents a mutable node inside a [`LinkedList`].
pub struct ListNode<'list> {
    prev: Option<NonNull<usize>>,
    head: Option<NonNull<usize>>,
    _lifetime: PhantomData<&'list mut LinkedList>,
}

impl ListNode<'_> {
    /// Remove this node from the [`LinkedList`].
    pub fn pop(&self) -> Option<NonNull<usize>> {
        // if there's a node after this one, link `prev` and `head.next`
        if let Some((prev, head)) = self.prev.zip(self.head) {
            // SAFETY
            // We checked if both pointers are non-null.
            unsafe { *prev.as_ptr() = *head.as_ptr() }
        }

        self.head
    }

    /// Returns the pointer to the address of the next node.
    pub fn as_ptr(&self) -> Option<NonNull<usize>> {
        self.head
    }
}

/// Iterator over all [`ListNode`]s of a single [`LinkedList`].
pub struct IterMut<'list> {
    prev: Option<NonNull<usize>>,
    head: Option<NonNull<usize>>,
    _lifetime: PhantomData<&'list mut LinkedList>,
}

impl<'list> Iterator for IterMut<'list> {
    type Item = ListNode<'list>;

    fn next(&mut self) -> Option<Self::Item> {
        // check if we reached the end
        let head = self.head?.as_ptr();

        // create the list node
        let node = ListNode {
            head: self.head,
            prev: self.prev,
            _lifetime: PhantomData,
        };

        // move one element forward
        self.head = self.prev;
        self.head = unsafe { Some(NonNull::new_unchecked(*head as *mut _)) };

        // return the new node
        Some(node)
    }
}
