use std::iter::IntoIterator;

////////////////////////////////////////////////////////////////////////////////
// List implementation

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Box<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    fn new() -> Self {
        List { head: None }
    }

    // TODO: Figure out how to return self.
    fn push(&mut self, elem: T) {
        self.head = Some(Box::new(Node { elem: elem, next: self.head.take() }));
    }

    fn pop(&mut self) -> Option<T> {
        // Option<Box<Node<T>>>
        self.head.take().map(|node| { // Box<Node<T>>
            let node = *node;
            self.head = node.next;
            node.elem
        })
    }

    fn peek(&self) -> Option<&T> {
        self.head.as_ref().map(|node| { &node.elem })
    }

    fn peek_mut(&mut self) -> Option<&mut T> {
        self.head.as_mut().map(|node| { &mut node.elem })
    }

    fn iter(&self) -> Iter<T> {
        Iter { link: &self.head }
    }

    fn iter_mut(&mut self) -> IterMut<T> {
        IterMut { next: Some(&mut self.head) }
    }
}

////////////////////////////////////////////////////////////////////////////////
// By-reference Iter

pub struct Iter<'a, T: 'a> {
    link: &'a Link<T>,
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        self.link.as_ref().map(|node| {
            self.link = &node.next;
            &node.elem
        })
    }
}

////////////////////////////////////////////////////////////////////////////////
// Mutable by-reference Iter

// FAILED ATTEMPT
//pub struct IterMut<'a, T: 'a> {
//    link: &'a mut Link<T>,
//}
//
//impl<'a, T> Iterator for IterMut<'a, T> {
//    type Item = &'a mut T;
//
//    fn next(&mut self) -> Option<Self::Item> {
//        self.link.as_mut().take().map(|node| {
//            self.link = &mut node.next;
//            &mut node.elem
//        })
//    }
//}

// KEY INSIGHT in working implementation:
// * On iterator initialization, the iterator lifetime is 'a.
// * On #next(), the lifetime of the iterator needs to be independent of 'a for the iterator to be
//   usable past one invocation of next(). But this independent lifetime prevents it from returning
//   something with lifetime 'a UNLESS it stores something else that it owns (ie. that it can
//   modify) that contains a pointer with lifetime 'a.
//
// Much easier way to implement is Gankro's version
//   http://cglab.ca/~abeinges/blah/too-many-lists/book/second-iter-mut.html
// which stores Option<&'a Node> instead.

pub struct IterMut<'a, T: 'a> {
    next: Option<&'a mut Link<T>>,  // (Link<T> = Option<Box<Node<T>>>)
}

impl<'a, T> Iterator for IterMut<'a, T> {
    type Item = &'a mut T;

    fn next(&mut self) -> Option<Self::Item> {
        self.next.take().and_then(|link| {
            // link has type: &mut Link<T>  (Link<T> = Option<Box<Node<T>>>)
            link.as_mut().map(|node| {
                // Need to deref the box first so that rust can infer that we are mut-borrowing
                // disjoint fields of a struct. Without this line, both times, we would be
                // mut-borrowing the box directly, and boxes are opaque to the compiler. See
                // https://users.rust-lang.org/t/trouble-implementing-linked-list-mutable-iterator-variant-based-on-gankros-tutorial/2357
                // for more info.
                let node = &mut **node;
                self.next = Some(&mut node.next);
                &mut node.elem
            })
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
        self.list.pop()
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

impl<'a, T> IntoIterator for &'a mut List<T> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> IterMut<'a, T> {
        self.iter_mut()
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod test {
    use super::List;

    #[test]
    fn test_push_pop() {
        let mut list = List::new();
        list.push(1);
        assert_eq!(Some(&1), list.peek());
        list.push(2);
        assert_eq!(Some(&2), list.peek());
        list.push(3);
        assert_eq!(Some(&3), list.peek());
        assert_eq!(Some(3), list.pop());
        assert_eq!(Some(2), list.pop());
        assert_eq!(Some(1), list.pop());
        assert_eq!(None, list.pop());
    }

    #[test]
    fn test_into_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut i = 3;
        for val in list {
            assert_eq!(i, val);
            i -= 1;
        }
    }

    #[test]
    fn test_iter() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut iter = list.iter();
        assert_eq!(Some(&3), iter.next());
        assert_eq!(Some(&2), iter.next());
        assert_eq!(Some(&1), iter.next());
        assert_eq!(None, iter.next());
        let mut i = 3;
        for val in &list {
            assert_eq!(i, *val);
            i -= 1;
        }
    }

    #[test]
    fn test_iter_mut() {
        let mut list = List::new();
        list.push(1);
        list.push(2);
        list.push(3);
        let mut i = 3;
        for val in list.iter_mut() {
            assert_eq!(i, *val);
            *val = 3 - i;
            i -= 1;
        }
        let mut i = 0;
        for val in &mut list {
            assert_eq!(i, *val);
            *val = 3 - i;
            i += 1;
        }
        let mut i = 3;
        for val in &list {
            assert_eq!(i, *val);
            i -= 1;
        }
    }
}
