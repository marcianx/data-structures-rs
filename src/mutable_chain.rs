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
}
