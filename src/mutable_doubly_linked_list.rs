/// This is a doubly-linked list implementation with forward pointers being owners and backward
/// pointers being unsafe pointers. The goal is to write the unsafe code involving the backward
/// links as similarly as the safe code for the forward links to avoid rampant stupidity.

use std::borrow::Borrow;
use std::borrow::BorrowMut;
use std::iter::IntoIterator;
use std::ptr;

////////////////////////////////////////////////////////////////////////////////
// List implementation

pub struct List<T> {
    head: Link<T>,
    tail: LinkPtr<T>,
}

type Link<T> = Option<Box<Node<T>>>;
type LinkPtr<T> = *const Node<T>;

struct Node<T> {
    elem: T,
    next: Link<T>,
    prev: LinkPtr<T>,
}

////////////////////////////////////////////////////////////
// Referential equality helpers
//
// While Option<&T> is Copy, Option<&mut T> is not, so taking Options by reference for consistency.

fn eq_ref_opt<T>(ref1: &T, opt_ref2: &Option<&T>) -> bool {
    match opt_ref2 {
        // Need to convert to *const for referential equality.
        &Some(ref ref2) => ref1 as *const T == *ref2 as *const T,
        &None => false
    }
}

fn eq_mut_ref_opt<T>(ref1: &mut T, opt_ref2: &Option<&mut T>) -> bool {
    match opt_ref2 {
        // Need to convert to *const for referential equality.
        &Some(ref ref2) => ref1 as *const T == *ref2 as *const T,
        &None => false
    }
}

////////////////////////////////////////////////////////////
// Box to ptr conversion

trait BoxHelpers<T> {
    fn to_ptr(&self) -> *const T;
}

impl<T> BoxHelpers<T> for Box<T> {
    fn to_ptr(&self) -> *const T {
        Borrow::<T>::borrow(self) as *const T
    }
}

////////////////////////////////////////////////////////////
/// Doubly-linked list

impl<T> List<T> {
    pub fn new() -> Self {
        List { head: None, tail: ptr::null() }
    }

    // PUSH
    pub fn push_front(&mut self, elem: T) {
        let mut node_box = Box::new(Node {
            elem: elem,
            next: self.head.take(),
            prev: ptr::null()
        });
        let node_ptr = node_box.to_ptr();
        match node_box.next.as_mut() { // What self.head used to be before it was take()n above.
            None => self.tail = node_ptr,
            Some(old_head_ref) => old_head_ref.prev = node_ptr
        }
        self.head = Some(node_box);
    }

    pub fn push_back(&mut self, elem: T) {
        let node_box = Box::new(Node {
            elem: elem,
            next: None,
            prev: self.tail
        });
        let node_ptr = node_box.to_ptr();
        if self.tail == ptr::null() {
            self.head = Some(node_box);
        } else {
            unsafe { (*(self.tail as *mut Node<T>)).next = Some(node_box) }
        }
        self.tail = node_ptr;
    }

    // POP
    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|node_box| {
            let node = *node_box;
            self.head = node.next;
            // INCOMPLETE handling of pointer part, causing test_push_pop_crash() to panic
            //if self.head.is_none() {
            //    self.tail = ptr::null()
            //}
            // CORRECT handling of pointer part!
            match self.head.as_mut() {
                None => self.tail = ptr::null(),
                Some(node_box) => node_box.prev = ptr::null()
            }
            node.elem
        })
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.tail == ptr::null() {
            return None;
        }
        self.tail = unsafe { (*self.tail).prev };
        let node_opt = if self.tail == ptr::null() {
                self.head.take()
            } else {
                unsafe { (*(self.tail as *mut Node<T>)).next.take() }
            };
        Some(node_opt.unwrap().elem) // Ideally, should use unwrap_unchecked().
    }

    // PEEK
    pub fn peek_front(&self) -> Option<&T> {
        self.head.as_ref().map(|node| { &node.elem })
    }

    pub fn peek_back(&self) -> Option<&T> {
        if self.tail == ptr::null() {
            None
        } else {
            Some(unsafe { &(*self.tail).elem })
        }
    }

    // PEEK MUT
    pub fn peek_front_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| { &mut node.elem })
    }

    pub fn peek_back_mut(&mut self) -> Option<&mut T> {
        if self.tail == ptr::null() {
            None
        } else {
            Some(unsafe { &mut (*(self.tail as *mut Node<T>)).elem })
        }
    }

    // ITER
    pub fn iter(&self) -> Iter<T> {
        Iter {
            front_link: self.head.as_ref().map(|node_box| node_box.borrow()),
            back_link: if self.tail == ptr::null() { None } else {
                Some(unsafe { &*self.tail })
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// By-reference Iter

pub struct Iter<'a, T: 'a> {
    front_link: Option<&'a Node<T>>,
    back_link: Option<&'a Node<T>>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.front_link.take().map(|node_ref| {
            if eq_ref_opt(node_ref, &self.back_link) { // If both ends collide, be DONE!
                self.front_link = None;
                self.back_link = None;
            } else {
                self.front_link = node_ref.next.as_ref().map(|node_box| node_box.borrow())
            }
            &node_ref.elem
        })
    }
}

impl<'a, T> DoubleEndedIterator for Iter<'a, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.back_link.take().map(|node_ref| {
            if eq_ref_opt(node_ref, &self.front_link) { // If both ends collide, be DONE!
                self.front_link = None;
                self.back_link = None;
            } else {
                self.back_link = Some(unsafe { &*node_ref.prev })
            }
            &node_ref.elem
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// IntoIterator

pub struct ListIntoIterator<T> {
    list: List<T>,
}

impl<T> Iterator for ListIntoIterator<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop_front()
    }
}

impl<T> DoubleEndedIterator for ListIntoIterator<T> {
    fn next_back(&mut self) -> Option<T> {
        self.list.pop_back()
    }
}

impl<T> IntoIterator for List<T> {
    type Item = T;
    type IntoIter = ListIntoIterator<T>;

    fn into_iter(self) -> ListIntoIterator<T> {
        ListIntoIterator { list: self }
    }
}

impl<'a, T> IntoIterator for &'a List<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Iter<'a, T> {
        self.iter()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn test_push_pop_front() {
        let mut list = List::new();
        list.push_front(1);
        assert_eq!(Some(&1), list.peek_front());
        list.push_front(2);
        assert_eq!(Some(&2), list.peek_front());
        list.push_front(3);
        assert_eq!(Some(&3), list.peek_front());
        assert_eq!(Some(3), list.pop_front());
        assert_eq!(Some(2), list.pop_front());
        assert_eq!(Some(1), list.pop_front());
        assert_eq!(None, list.pop_front());
    }

    #[test]
    fn test_push_pop_back() {
        let mut list = List::new();
        list.push_back(1);
        assert_eq!(Some(&1), list.peek_back());
        list.push_back(2);
        assert_eq!(Some(&2), list.peek_back());
        list.push_back(3);
        assert_eq!(Some(&3), list.peek_back());
        assert_eq!(Some(3), list.pop_back());
        assert_eq!(Some(2), list.pop_back());
        assert_eq!(Some(1), list.pop_back());
        assert_eq!(None, list.pop_back());
    }

    #[test]
    fn test_push_pop_both() {
        let mut list = List::new();
        list.push_back(2);
        assert_eq!(Some(&2), list.peek_back());
        list.push_front(1);
        assert_eq!(Some(&1), list.peek_front());
        assert_eq!(Some(&2), list.peek_back());
        list.push_back(3);
        assert_eq!(Some(&1), list.peek_front());
        assert_eq!(Some(&3), list.peek_back());
        assert_eq!(Some(3), list.pop_back());
        assert_eq!(Some(1), list.pop_front());
        assert_eq!(Some(2), list.pop_front());
        assert_eq!(None, list.pop_front());
        assert_eq!(None, list.pop_back());
    }

    #[test]
    fn test_push_pop_crash() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        let mut i = 3;
        for val in list {
            assert_eq!(i, val);
            i -= 1;
        }

        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        assert_eq!(Some(1), list.pop_front());
        assert_eq!(Some(3), list.pop_back());
        assert_eq!(Some(2), list.pop_back());
        assert_eq!(None, list.pop_front());
        assert_eq!(None, list.pop_back());
    }

    #[test]
    fn test_into_iter() {
        let mut list = List::new();
        list.push_front(1);
        list.push_front(2);
        list.push_front(3);
        let mut i = 3;
        for val in list {
            assert_eq!(i, val);
            i -= 1;
        }

        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        let mut iter = list.into_iter();
        assert_eq!(Some(1), iter.next());
        assert_eq!(Some(3), iter.next_back());
        assert_eq!(Some(2), iter.next_back());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());
    }

    #[test]
    fn test_iter() {
        let mut list = List::new();
        list.push_back(1);
        list.push_back(2);
        list.push_back(3);
        let mut iter = list.iter();
        assert_eq!(Some(&1), iter.next());
        assert_eq!(Some(&3), iter.next_back());
        assert_eq!(Some(&2), iter.next_back());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());

        let mut iter = list.iter();
        assert_eq!(Some(&3), iter.next_back());
        assert_eq!(Some(&1), iter.next());
        assert_eq!(Some(&2), iter.next());
        assert_eq!(None, iter.next());
        assert_eq!(None, iter.next_back());

        let mut i = 1;
        for val in &list {
            assert_eq!(i, *val);
            i += 1;
        }
    }
}

