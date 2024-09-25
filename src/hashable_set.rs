use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

// Define a wrapper around HashSet that implements Hash and Eq
#[derive(Debug, Default, Clone)]
pub struct HashableSet<T>(HashSet<T>);

impl<T> Deref for HashableSet<T> {
    type Target = HashSet<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for HashableSet<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T: Eq + Hash + Ord> HashableSet<T> {
    #[allow(unused)]
    fn iter_sorted(&self) -> std::vec::IntoIter<&T> {
        let mut sorted_elements: Vec<_> = self.0.iter().collect();
        sorted_elements.sort();
        sorted_elements.into_iter()
    }
}

impl<T: Eq + Hash + Ord> FromIterator<T> for HashableSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self(HashSet::from_iter(iter))
    }
}

impl<T: Eq + Hash + Ord> PartialEq for HashableSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl<T: Eq + Hash + Ord> Eq for HashableSet<T> {}

impl<T: Eq + Hash + Ord> Hash for HashableSet<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Create a sorted Vec from the set
        let mut sorted_elements: Vec<_> = self.0.iter().collect();
        sorted_elements.sort();
        // Hash each element in sorted order
        for element in sorted_elements {
            element.hash(state);
        }
    }
}
