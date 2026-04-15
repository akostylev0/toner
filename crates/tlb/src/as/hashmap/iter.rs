use super::aug::{Hashmap, HashmapAugE, HashmapE, HashmapNode};
use bitvec::bitvec;
use bitvec::order::Msb0;
use bitvec::prelude::BitVec;

/// Iterator over `(BitVec<u8, Msb0>, &T)` pairs of a [`HashmapE`] in key order.
///
/// Created by [`HashmapE::iter`].
pub struct HashmapIter<'a, T, E = ()> {
    stack: Vec<(BitVec<u8, Msb0>, &'a Hashmap<T, E>)>,
}

impl<'a, T, E> Iterator for HashmapIter<'a, T, E> {
    type Item = (BitVec<u8, Msb0>, &'a T);

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

impl<'a, T, E> IntoIterator for &'a Hashmap<T, E> {
    type Item = (BitVec<u8, Msb0>, &'a T);
    type IntoIter = HashmapIter<'a, T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        HashmapIter {
            stack: vec![(bitvec![u8, Msb0;], self)],
        }
    }
}

impl<'a, T, E> IntoIterator for &'a HashmapE<T, E> {
    type Item = (BitVec<u8, Msb0>, &'a T);
    type IntoIter = HashmapIter<'a, T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            HashmapE::Empty => HashmapIter { stack: Vec::new() },
            HashmapE::Root(root) => root.into_iter(),
        }
    }
}

impl<'a, T, E> IntoIterator for &'a HashmapAugE<T, E> {
    type Item = (BitVec<u8, Msb0>, &'a T);
    type IntoIter = HashmapIter<'a, T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.m.iter()
    }
}

pub struct HashmapIterMut<'a, T, E = ()> {
    stack: Vec<(BitVec<u8, Msb0>, &'a mut Hashmap<T, E>)>,
}

impl<'a, T, E> Iterator for HashmapIterMut<'a, T, E> {
    type Item = (BitVec<u8, Msb0>, &'a mut T);

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

impl<'a, T, E> IntoIterator for &'a mut Hashmap<T, E> {
    type Item = (BitVec<u8, Msb0>, &'a mut T);
    type IntoIter = HashmapIterMut<'a, T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        HashmapIterMut {
            stack: vec![(bitvec![u8, Msb0;], self)],
        }
    }
}

impl<'a, T, E> IntoIterator for &'a mut HashmapE<T, E> {
    type Item = (BitVec<u8, Msb0>, &'a mut T);
    type IntoIter = HashmapIterMut<'a, T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            HashmapE::Empty => HashmapIterMut { stack: Vec::new() },
            HashmapE::Root(root) => root.into_iter(),
        }
    }
}

impl<'a, T, E> IntoIterator for &'a mut HashmapAugE<T, E> {
    type Item = (BitVec<u8, Msb0>, &'a mut T);
    type IntoIter = HashmapIterMut<'a, T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.m.iter_mut()
    }
}

pub struct HashmapIntoIter<T, E = ()> {
    stack: Vec<(BitVec<u8, Msb0>, Hashmap<T, E>)>,
}

impl<T, E> Iterator for HashmapIntoIter<T, E> {
    type Item = (BitVec<u8, Msb0>, T);

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

impl<T, E> IntoIterator for Hashmap<T, E> {
    type Item = (BitVec<u8, Msb0>, T);
    type IntoIter = HashmapIntoIter<T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        HashmapIntoIter {
            stack: vec![(bitvec![u8, Msb0;], self)],
        }
    }
}

impl<T, E> IntoIterator for HashmapE<T, E> {
    type Item = (BitVec<u8, Msb0>, T);
    type IntoIter = HashmapIntoIter<T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        match self {
            HashmapE::Empty => HashmapIntoIter { stack: Vec::new() },
            HashmapE::Root(root) => root.into_iter(),
        }
    }
}

impl<T, E> IntoIterator for HashmapAugE<T, E> {
    type Item = (BitVec<u8, Msb0>, T);
    type IntoIter = HashmapIntoIter<T, E>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.m.into_iter()
    }
}
