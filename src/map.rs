// Copyright 2017 Serde Developers
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! A map of a key to a value.
//!
//! By default the map is backed by a [`BTreeMap`]. Enable the `preserve_order`
//! feature of toml-rs to use [`LinkedHashMap`] instead.
//!
//! [`BTreeMap`]: https://doc.rust-lang.org/std/collections/struct.BTreeMap.html
//! [`LinkedHashMap`]: https://docs.rs/linked-hash-map/*/linked_hash_map/struct.LinkedHashMap.html

use serde::{de, ser};
use std::borrow::Borrow;
use std::fmt::{self, Debug};
use std::hash::Hash;
use std::iter::FromIterator;
use std::marker::PhantomData;
use std::ops;

#[cfg(not(feature = "preserve_order"))]
use std::collections::{btree_map, BTreeMap};

#[cfg(feature = "preserve_order")]
use linked_hash_map::{self, LinkedHashMap};

/// Represents a JSON key/value type.
#[derive(Clone, PartialEq)]
pub struct Map<K: Ord + Hash, V> {
    map: MapImpl<K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type MapImpl<K, V> = BTreeMap<K, V>;
#[cfg(feature = "preserve_order")]
type MapImpl<K, V> = LinkedHashMap<K, V>;

impl<K: Ord + Hash, V> Map<K, V> {
    /// Makes a new empty Map.
    #[inline]
    pub fn new() -> Self {
        Map {
            map: MapImpl::new(),
        }
    }

    #[cfg(not(feature = "preserve_order"))]
    /// Makes a new empty Map with the given initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        // does not support with_capacity
        let _ = capacity;
        Map {
            map: BTreeMap::new(),
        }
    }

    #[cfg(feature = "preserve_order")]
    /// Makes a new empty Map with the given initial capacity.
    #[inline]
    pub fn with_capacity(capacity: usize) -> Self {
        Map {
            map: LinkedHashMap::with_capacity(capacity),
        }
    }

    /// Clears the map, removing all values.
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear()
    }

    /// Returns a reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn get<Q: ?Sized>(&self, key: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Ord + Eq + Hash,
    {
        self.map.get(key)
    }

    /// Returns true if the map contains a value for the specified key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn contains_key<Q: ?Sized>(&self, key: &Q) -> bool
    where
        K: Borrow<Q>,
        Q: Ord + Eq + Hash,
    {
        self.map.contains_key(key)
    }

    /// Returns a mutable reference to the value corresponding to the key.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn get_mut<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Ord + Eq + Hash,
    {
        self.map.get_mut(key)
    }

    /// Inserts a key-value pair into the map.
    ///
    /// If the map did not have this key present, `None` is returned.
    ///
    /// If the map did have this key present, the value is updated, and the old
    /// value is returned. The key is not updated, though; this matters for
    /// types that can be `==` without being identical.
    #[inline]
    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.map.insert(k, v)
    }

    /// Removes a key from the map, returning the value at the key if the key
    /// was previously in the map.
    ///
    /// The key may be any borrowed form of the map's key type, but the ordering
    /// on the borrowed form *must* match the ordering on the key type.
    #[inline]
    pub fn remove<Q: ?Sized>(&mut self, key: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: Ord + Eq + Hash,
    {
        self.map.remove(key)
    }

    /// Gets the given key's corresponding entry in the map for in-place
    /// manipulation.
    pub fn entry<S>(&mut self, key: S) -> Entry<'_, K, V>
    where
        S: Into<K>,
    {
        #[cfg(feature = "preserve_order")]
        use linked_hash_map::Entry as EntryImpl;
        #[cfg(not(feature = "preserve_order"))]
        use std::collections::btree_map::Entry as EntryImpl;

        match self.map.entry(key.into()) {
            EntryImpl::Vacant(vacant) => Entry::Vacant(VacantEntry { vacant }),
            EntryImpl::Occupied(occupied) => Entry::Occupied(OccupiedEntry { occupied }),
        }
    }

    /// Returns the number of elements in the map.
    #[inline]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// Returns true if the map contains no elements.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    /// Gets an iterator over the entries of the map.
    #[inline]
    pub fn iter(&self) -> Iter<'_, K, V> {
        Iter {
            iter: self.map.iter(),
        }
    }

    /// Gets a mutable iterator over the entries of the map.
    #[inline]
    pub fn iter_mut(&mut self) -> IterMut<'_, K, V> {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }

    /// Gets an iterator over the keys of the map.
    #[inline]
    pub fn keys(&self) -> Keys<'_, K, V> {
        Keys {
            iter: self.map.keys(),
        }
    }

    /// Gets an iterator over the values of the map.
    #[inline]
    pub fn values(&self) -> Values<'_, K, V> {
        Values {
            iter: self.map.values(),
        }
    }
}

impl<K: Ord + Hash, V> Default for Map<K, V> {
    #[inline]
    fn default() -> Self {
        Map {
            map: MapImpl::new(),
        }
    }
}

/// Access an element of this map. Panics if the given key is not present in the
/// map.
impl<'a, K, V, Q: ?Sized> ops::Index<&'a Q> for Map<K, V>
where
    K: Ord + Hash + Borrow<Q>,
    Q: Ord + Eq + Hash,
{
    type Output = V;

    fn index(&self, index: &Q) -> &V {
        self.map.index(index)
    }
}

/// Mutably access an element of this map. Panics if the given key is not
/// present in the map.
impl<'a, K, V, Q: ?Sized> ops::IndexMut<&'a Q> for Map<K, V>
where
    K: Ord + Hash + Borrow<Q>,
    Q: Ord + Eq + Hash,
{
    fn index_mut(&mut self, index: &Q) -> &mut V {
        self.map.get_mut(index).expect("no entry found for key")
    }
}

impl<K: Ord + Hash + Debug, V: Debug> Debug for Map<K, V> {
    #[inline]
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        self.map.fmt(formatter)
    }
}

impl<K: Ord + Hash + ser::Serialize, V: ser::Serialize> ser::Serialize for Map<K, V> {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        use serde::ser::SerializeMap;
        let mut map = serializer.serialize_map(Some(self.len()))?;
        for (k, v) in self {
            map.serialize_key(k)?;
            map.serialize_value(v)?;
        }
        map.end()
    }
}

impl<'de, K, V> de::Deserialize<'de> for Map<K, V>
where
    K: Ord + Hash + de::Deserialize<'de>,
    V: de::Deserialize<'de>,
{
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        struct Visitor<VK, VV>(PhantomData<(VK, VV)>);

        impl<'de, VK, VV> de::Visitor<'de> for Visitor<VK, VV>
        where
            VK: Hash + Ord + de::Deserialize<'de>,
            VV: de::Deserialize<'de>,
        {
            type Value = Map<VK, VV>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a map")
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                Ok(Map::new())
            }

            #[inline]
            fn visit_map<V>(self, mut visitor: V) -> Result<Self::Value, V::Error>
            where
                V: de::MapAccess<'de>,
            {
                let mut values = Map::new();

                while let Some((key, value)) = visitor.next_entry()? {
                    values.insert(key, value);
                }

                Ok(values)
            }
        }

        deserializer.deserialize_map(Visitor(PhantomData::<(K, V)>))
    }
}

impl<K: Ord + Hash, V> FromIterator<(K, V)> for Map<K, V> {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = (K, V)>,
    {
        Map {
            map: FromIterator::from_iter(iter),
        }
    }
}

impl<K: Ord + Hash, V> Extend<(K, V)> for Map<K, V> {
    fn extend<T>(&mut self, iter: T)
    where
        T: IntoIterator<Item = (K, V)>,
    {
        self.map.extend(iter);
    }
}

macro_rules! delegate_iterator {
    (($name:ident [$($generics_decl:tt)*], $($generics:tt)*) => $item:ty) => {
        impl <$($generics_decl)*> Iterator for $name $($generics)* {
            type Item = $item;
            #[inline]
            fn next(&mut self) -> Option<Self::Item> {
                self.iter.next()
            }
            #[inline]
            fn size_hint(&self) -> (usize, Option<usize>) {
                self.iter.size_hint()
            }
        }

        impl <$($generics_decl)*> DoubleEndedIterator for $name $($generics)* {
            #[inline]
            fn next_back(&mut self) -> Option<Self::Item> {
                self.iter.next_back()
            }
        }

        impl <$($generics_decl)*> ExactSizeIterator for $name $($generics)* {
            #[inline]
            fn len(&self) -> usize {
                self.iter.len()
            }
        }
    }
}

//////////////////////////////////////////////////////////////////////////////

/// A view into a single entry in a map, which may either be vacant or occupied.
/// This enum is constructed from the [`entry`] method on [`Map`].
///
/// [`entry`]: struct.Map.html#method.entry
/// [`Map`]: struct.Map.html
pub enum Entry<'a, K: Ord + Hash, V> {
    /// A vacant Entry.
    Vacant(VacantEntry<'a, K, V>),
    /// An occupied Entry.
    Occupied(OccupiedEntry<'a, K, V>),
}

/// A vacant Entry. It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct VacantEntry<'a, K: Ord + Hash, V> {
    vacant: VacantEntryImpl<'a, K, V>,
}

/// An occupied Entry. It is part of the [`Entry`] enum.
///
/// [`Entry`]: enum.Entry.html
pub struct OccupiedEntry<'a, K: Ord + Hash, V> {
    occupied: OccupiedEntryImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type VacantEntryImpl<'a, K, V> = btree_map::VacantEntry<'a, K, V>;
#[cfg(feature = "preserve_order")]
type VacantEntryImpl<'a, K, V> = linked_hash_map::VacantEntry<'a, K, V>;

#[cfg(not(feature = "preserve_order"))]
type OccupiedEntryImpl<'a, K, V> = btree_map::OccupiedEntry<'a, K, V>;
#[cfg(feature = "preserve_order")]
type OccupiedEntryImpl<'a, K, V> = linked_hash_map::OccupiedEntry<'a, K, V>;

impl<'a, K: Ord + Hash, V> Entry<'a, K, V> {
    /// Returns a reference to this entry's key.
    pub fn key(&self) -> &K {
        match *self {
            Entry::Vacant(ref e) => e.key(),
            Entry::Occupied(ref e) => e.key(),
        }
    }

    /// Ensures a value is in the entry by inserting the default if empty, and
    /// returns a mutable reference to the value in the entry.
    pub fn or_insert(self, default: V) -> &'a mut V {
        match self {
            Entry::Vacant(entry) => entry.insert(default),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }

    /// Ensures a value is in the entry by inserting the result of the default
    /// function if empty, and returns a mutable reference to the value in the
    /// entry.
    pub fn or_insert_with<F>(self, default: F) -> &'a mut V
    where
        F: FnOnce() -> V,
    {
        match self {
            Entry::Vacant(entry) => entry.insert(default()),
            Entry::Occupied(entry) => entry.into_mut(),
        }
    }
}

impl<'a, K: Ord + Hash, V> VacantEntry<'a, K, V> {
    /// Gets a reference to the key that would be used when inserting a value
    /// through the VacantEntry.
    #[inline]
    pub fn key(&self) -> &K {
        self.vacant.key()
    }

    /// Sets the value of the entry with the VacantEntry's key, and returns a
    /// mutable reference to it.
    #[inline]
    pub fn insert(self, value: V) -> &'a mut V {
        self.vacant.insert(value)
    }
}

impl<'a, K: Ord + Hash, V> OccupiedEntry<'a, K, V> {
    /// Gets a reference to the key in the entry.
    #[inline]
    pub fn key(&self) -> &K {
        self.occupied.key()
    }

    /// Gets a reference to the value in the entry.
    #[inline]
    pub fn get(&self) -> &V {
        self.occupied.get()
    }

    /// Gets a mutable reference to the value in the entry.
    #[inline]
    pub fn get_mut(&mut self) -> &mut V {
        self.occupied.get_mut()
    }

    /// Converts the entry into a mutable reference to its value.
    #[inline]
    pub fn into_mut(self) -> &'a mut V {
        self.occupied.into_mut()
    }

    /// Sets the value of the entry with the `OccupiedEntry`'s key, and returns
    /// the entry's old value.
    #[inline]
    pub fn insert(&mut self, value: V) -> V {
        self.occupied.insert(value)
    }

    /// Takes the value of the entry out of the map, and returns it.
    #[inline]
    pub fn remove(self) -> V {
        self.occupied.remove()
    }
}

//////////////////////////////////////////////////////////////////////////////

impl<'a, K: Ord + Hash, V> IntoIterator for &'a Map<K, V> {
    type Item = (&'a K, &'a V);
    type IntoIter = Iter<'a, K, V>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        Iter {
            iter: self.map.iter(),
        }
    }
}

/// An iterator over a toml::Map's entries.
pub struct Iter<'a, K: Ord + Hash, V> {
    iter: IterImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type IterImpl<'a, K, V> = btree_map::Iter<'a, K, V>;
#[cfg(feature = "preserve_order")]
type IterImpl<'a, K, V> = linked_hash_map::Iter<'a, K, V>;

delegate_iterator!((Iter['a, K: Ord + Hash, V], <'a, K, V>) => (&'a K, &'a V));

//////////////////////////////////////////////////////////////////////////////

impl<'a, K: Ord + Hash, V> IntoIterator for &'a mut Map<K, V> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = IterMut<'a, K, V>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IterMut {
            iter: self.map.iter_mut(),
        }
    }
}

/// A mutable iterator over a toml::Map's entries.
pub struct IterMut<'a, K: Ord + Hash, V> {
    iter: IterMutImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type IterMutImpl<'a, K, V> = btree_map::IterMut<'a, K, V>;
#[cfg(feature = "preserve_order")]
type IterMutImpl<'a, K, V> = linked_hash_map::IterMut<'a, K, V>;

delegate_iterator!((IterMut['a, K: Ord + Hash, V], <'a, K, V>) => (&'a K, &'a mut V));

//////////////////////////////////////////////////////////////////////////////

impl<K: Ord + Hash, V> IntoIterator for Map<K, V> {
    type Item = (K, V);
    type IntoIter = IntoIter<K, V>;
    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            iter: self.map.into_iter(),
        }
    }
}

/// An owning iterator over a toml::Map's entries.
pub struct IntoIter<K: Ord + Hash, V> {
    iter: IntoIterImpl<K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type IntoIterImpl<K, V> = btree_map::IntoIter<K, V>;
#[cfg(feature = "preserve_order")]
type IntoIterImpl<K, V> = linked_hash_map::IntoIter<K, V>;

delegate_iterator!((IntoIter[K: Ord + Hash, V], <K, V>) => (K, V));

//////////////////////////////////////////////////////////////////////////////

/// An iterator over a toml::Map's keys.
pub struct Keys<'a, K: Ord + Hash, V> {
    iter: KeysImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type KeysImpl<'a, K, V> = btree_map::Keys<'a, K, V>;
#[cfg(feature = "preserve_order")]
type KeysImpl<'a, K, V> = linked_hash_map::Keys<'a, K, V>;

delegate_iterator!((Keys['a, K: Ord + Hash, V], <'a, K, V>) => &'a K);

//////////////////////////////////////////////////////////////////////////////

/// An iterator over a toml::Map's values.
pub struct Values<'a, K: Ord + Hash, V> {
    iter: ValuesImpl<'a, K, V>,
}

#[cfg(not(feature = "preserve_order"))]
type ValuesImpl<'a, K, V> = btree_map::Values<'a, K, V>;
#[cfg(feature = "preserve_order")]
type ValuesImpl<'a, K, V> = linked_hash_map::Values<'a, K, V>;

delegate_iterator!((Values['a, K: Ord + Hash, V], <'a, K, V>) => &'a V);
