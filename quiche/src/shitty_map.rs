use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::btree_map::Keys;
use std::collections::BTreeMap;
use std::convert::Infallible;
use std::mem;
use std::slice::Iter;

use smallvec::smallvec;
use smallvec::SmallVec;

/// documentation lol
pub type ShittySet<T> = ShittyMap<T, ()>;

/// documentation lol
pub enum ShittyMap<K, V> {
    /// documentation lol
    Empty,
    /// documentation lol
    Vec(SmallVec<[(K, V); 10]>),
    /// documentation lol
    Map(BTreeMap<K, V>),
}

impl<K, V> ShittyMap<K, V> {
    /// documentation lol
    pub fn new() -> Self {
        Self::Empty
    }

    /// documentation lol
    pub fn insert(&mut self, key: K, value: V) -> Option<V>
    where
        K: Ord,
    {
        match self {
            Self::Empty => {
                *self = Self::Vec(smallvec![(key.into(), value)]);
                return None;
            },
            Self::Vec(data) => {
                for (e_key, e_val) in data.iter_mut() {
                    if key.borrow().cmp((*e_key).borrow()) == Ordering::Equal {
                        return Some(mem::replace(e_val, value));
                    }
                }

                if data.len() == data.inline_size() {
                    // fall through and upgrade to map
                } else {
                    data.push((key, value));
                    return None;
                }
            },
            Self::Map(map) => return map.insert(key, value),
        }

        match mem::replace(self, Self::Map(BTreeMap::new())) {
            Self::Vec(data) => match self {
                Self::Map(map) => {
                    for (e_key, e_val) in data.into_iter() {
                        map.insert(e_key, e_val);
                    }

                    None
                },
                _ => unreachable!(),
            },
            _ => unreachable!(),
        }
    }

    /// documentation lol
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        match self {
            Self::Empty => None,
            Self::Vec(data) => {
                let mut found_index = None;
                for (i, (e_key, _)) in data.iter_mut().enumerate() {
                    if key.borrow().cmp((*e_key).borrow()) == Ordering::Equal {
                        found_index = Some(i);
                        break;
                    }
                }

                if let Some(found_index) = found_index {
                    let (_, o_val) = data.swap_remove(found_index);
                    Some(o_val)
                } else {
                    None
                }
            },
            Self::Map(map) => map.remove(key),
        }
    }

    /// documentation lol
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        self.get(key).is_some()
    }

    /// documentation lol
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        match self {
            Self::Empty => None,
            Self::Vec(data) => {
                for (e_key, e_val) in data.iter() {
                    if key.borrow().cmp((*e_key).borrow()) == Ordering::Equal {
                        return Some(e_val);
                    }
                }

                None
            },
            Self::Map(map) => map.get(key),
        }
    }

    /// documentation lol
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q> + Ord,
        Q: Ord,
    {
        match self {
            Self::Empty => None,
            Self::Vec(data) => {
                for (e_key, e_val) in data.iter_mut() {
                    if key.borrow().cmp((*e_key).borrow()) == Ordering::Equal {
                        return Some(e_val);
                    }
                }

                None
            },
            Self::Map(map) => map.get_mut(key),
        }
    }

    /// documentation lol
    pub fn get_or_create<F>(&mut self, key: K, val_fn: F) -> &mut V
    where
        F: FnOnce() -> V,
        K: Ord + Copy,
    {
        self.get_or_create_result::<_, Infallible>(key, || Ok((val_fn)()))
            .unwrap()
    }

    /// documentation lol
    pub fn get_or_create_result<F, E>(
        &mut self, key: K, val_fn: F,
    ) -> Result<&mut V, E>
    where
        F: FnOnce() -> Result<V, E>,
        K: Ord + Copy,
    {
        let mut found_index = None;
        match self {
            Self::Empty => {
                *self = Self::Vec(smallvec![(key.into(), (val_fn)()?)]);
                match self {
                    Self::Vec(data) => {
                        let (_, e_val) = &mut data[0];
                        return Ok(e_val);
                    },
                    _ => unreachable!(),
                }
            },
            Self::Vec(data) => {
                for (i, (e_key, _)) in data.iter_mut().enumerate() {
                    if key.borrow().cmp((*e_key).borrow()) == Ordering::Equal {
                        found_index = Some(i);
                        break;
                    }
                }

                if found_index.is_none() {
                    if data.len() == data.inline_size() {
                        match mem::replace(self, Self::Map(BTreeMap::new())) {
                            Self::Vec(data) => match self {
                                Self::Map(map) => {
                                    for (e_key, e_val) in data.into_iter() {
                                        map.insert(e_key, e_val);
                                    }

                                    map.insert(key, (val_fn)()?);
                                    return Ok(map.get_mut(&key).unwrap());
                                },
                                _ => unreachable!(),
                            },
                            _ => unreachable!(),
                        }
                    } else {
                        data.push((key, (val_fn)()?));
                        found_index = Some(data.len() - 1);
                    }
                }
            },
            Self::Map(map) => match map.entry(key) {
                std::collections::btree_map::Entry::Vacant(v) => {
                    return Ok(v.insert((val_fn)()?));
                },
                std::collections::btree_map::Entry::Occupied(o) => {
                    return Ok(o.into_mut());
                },
            },
        }

        match (found_index, self) {
            (Some(found_index), Self::Vec(data)) => {
                let (_, e_val) = &mut data[found_index];
                Ok(e_val)
            },
            _ => unreachable!(),
        }
    }

    /// documentation lol
    pub fn len(&self) -> usize {
        match self {
            ShittyMap::Empty => 0,
            ShittyMap::Vec(v) => v.len(),
            ShittyMap::Map(m) => m.len(),
        }
    }

    /// documentation lol
    pub fn keys(&self) -> ShittyKeys<'_, K, V> {
        match self {
            Self::Empty => ShittyKeys::Empty,
            Self::Vec(v) => ShittyKeys::Vec(v.iter()),
            Self::Map(m) => ShittyKeys::Map(m.keys()),
        }
    }
}

impl<K, V> Default for ShittyMap<K, V> {
    fn default() -> Self {
        Self::Empty
    }
}

/// documentation lol
pub enum ShittyKeys<'a, K, V> {
    /// documentation lol
    Empty,
    /// documentation lol
    Vec(Iter<'a, (K, V)>),
    /// documentation lol
    Map(Keys<'a, K, V>),
}

impl<'a, K, V> Iterator for ShittyKeys<'a, K, V> {
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            ShittyKeys::Empty => None,
            ShittyKeys::Vec(v) => v.next().map(|(k, _)| k),
            ShittyKeys::Map(m) => m.next(),
        }
    }
}
