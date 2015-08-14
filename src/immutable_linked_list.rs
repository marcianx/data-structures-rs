use std::iter::IntoIterator;
use std::rc::Rc;

////////////////////////////////////////////////////////////////////////////////
// List implementation

pub struct List<T> {
    head: Link<T>,
}

type Link<T> = Option<Rc<Node<T>>>;

struct Node<T> {
    elem: T,
    next: Link<T>,
}

impl<T> List<T> {
    fn new() -> Self {
        List { head: None }
    }

    fn prepend(&self, elem: T) -> List<T> {
        List { head: Some(Rc::new(Node { elem: elem, next: self.head.clone() })) }
    }

    fn cons(elem: T, list: List<T>) -> List<T> { cons(elem, list) }

    fn tail(&self) -> List<T> {
        List { head: self.head.as_ref().and_then(|node_ref| node_ref.next.clone()) }
    }

    fn head(&self) -> Option<&T> {
        self.head.as_ref().map(|node_ref| &node_ref.elem)
    }

    fn iter(&self) -> Iter<T> {
        Iter { link: &self.head }
    }
}

fn cons<T>(elem: T, list: List<T>) -> List<T> {
    list.prepend(elem)
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
    use super::{List, cons};

    #[test]
    fn test_construction() {
        let mut list = List::new();
        list = list.prepend(1);
        assert_eq!(Some(&1), list.head());
        list = List::cons(2, list);
        assert_eq!(Some(&2), list.head());
        assert_eq!(Some(&1), list.tail().head());
        assert_eq!(None, list.tail().tail().head());
        list = cons(3, list);
        assert_eq!(Some(&3), list.head());
        assert_eq!(Some(&2), list.tail().head());
        assert_eq!(Some(&1), list.tail().tail().head());
        assert_eq!(None, list.tail().tail().tail().head());
        assert_eq!(None, list.tail().tail().tail().tail().head());
    }

    #[test]
    fn test_iter() {
        let mut list = List::new();
        list = cons(1, list);
        list = cons(2, list);
        list = cons(3, list);
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
}
