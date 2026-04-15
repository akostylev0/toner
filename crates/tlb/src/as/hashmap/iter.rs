use super::aug::{Hashmap, HashmapAugE, HashmapE, HashmapNode, Key};

/// Iterator over `(Key, &T)` pairs of a [`HashmapE`] in key order.
///
/// Created by [`HashmapE::iter`].
pub struct HashmapIter<'a, T, E = ()> {
    stack: Vec<(Key, &'a Hashmap<T, E>)>,
}

impl<'a, T, E> Iterator for HashmapIter<'a, T, E> {
    type Item = (Key, &'a T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (mut key, hashmap) = self.stack.pop()?;
            key.extend_from_bitslice(&hashmap.prefix);

            match &hashmap.node.node {
                HashmapNode::Leaf(value) => return Some((key, value)),
                HashmapNode::Fork([left, right]) => {
                    let mut right_key = key.clone();
                    right_key.push(true);
                    self.stack.push((right_key, right));

                    let mut left_key = key;
                    left_key.push(false);
                    self.stack.push((left_key, left));
                }
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower = self.stack.len();
        (lower, None)
    }
}

impl<'a, T, E> IntoIterator for &'a HashmapE<T, E> {
    type Item = (Key, &'a T);
    type IntoIter = HashmapIter<'a, T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        HashmapIter {
            stack: match self {
                HashmapE::Empty => Vec::new(),
                HashmapE::Root(root) => vec![(Key::new(), root)],
            },
        }
    }
}

impl<'a, T, E> IntoIterator for &'a HashmapAugE<T, E> {
    type Item = (Key, &'a T);
    type IntoIter = HashmapIter<'a, T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.m.iter()
    }
}

pub struct HashmapIterMut<'a, T, E = ()> {
    stack: Vec<(Key, &'a mut Hashmap<T, E>)>,
}

impl<'a, T, E> Iterator for HashmapIterMut<'a, T, E> {
    type Item = (Key, &'a mut T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (mut key, hashmap) = self.stack.pop()?;
            key.extend_from_bitslice(&hashmap.prefix);

            match &mut hashmap.node.node {
                HashmapNode::Leaf(value) => return Some((key, value)),
                HashmapNode::Fork([left, right]) => {
                    let mut right_key = key.clone();
                    right_key.push(true);
                    self.stack.push((right_key, right));

                    let mut left_key = key;
                    left_key.push(false);
                    self.stack.push((left_key, left));
                }
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower = self.stack.len();
        (lower, None)
    }
}

impl<'a, T, E> IntoIterator for &'a mut HashmapE<T, E> {
    type Item = (Key, &'a mut T);
    type IntoIter = HashmapIterMut<'a, T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        HashmapIterMut {
            stack: match self {
                HashmapE::Empty => Vec::new(),
                HashmapE::Root(root) => vec![(Key::new(), root)],
            },
        }
    }
}

impl<'a, T, E> IntoIterator for &'a mut HashmapAugE<T, E> {
    type Item = (Key, &'a mut T);
    type IntoIter = HashmapIterMut<'a, T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.m.iter_mut()
    }
}

pub struct HashmapIntoIter<T, E = ()> {
    stack: Vec<(Key, Hashmap<T, E>)>,
}

impl<T, E> Iterator for HashmapIntoIter<T, E> {
    type Item = (Key, T);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (mut key, hashmap) = self.stack.pop()?;
            key.extend_from_bitslice(&hashmap.prefix);

            match hashmap.node.node {
                HashmapNode::Leaf(value) => return Some((key, value)),
                HashmapNode::Fork([left, right]) => {
                    let mut right_key = key.clone();
                    right_key.push(true);
                    self.stack.push((right_key, *right));

                    let mut left_key = key;
                    left_key.push(false);
                    self.stack.push((left_key, *left));
                }
            }
        }
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let lower = self.stack.len();
        (lower, None)
    }
}

impl<T, E> IntoIterator for HashmapE<T, E> {
    type Item = (Key, T);
    type IntoIter = HashmapIntoIter<T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        HashmapIntoIter {
            stack: match self {
                HashmapE::Empty => Vec::new(),
                HashmapE::Root(root) => vec![(Key::new(), root)],
            },
        }
    }
}

impl<T, E> IntoIterator for HashmapAugE<T, E> {
    type Item = (Key, T);
    type IntoIter = HashmapIntoIter<T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.m.into_iter()
    }
}
