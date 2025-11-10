// Copyright 2025 the Spoonbender Authors
// SPDX-License-Identifier: Apache-2.0

//! Selection management for editing

use crate::entity_id::EntityId;
use std::collections::BTreeSet;
use std::sync::Arc;

/// A set of selected entities (points, paths, guides, etc.)
///
/// Uses Arc<BTreeSet> for efficient cloning and ordered iteration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Selection {
    inner: Arc<BTreeSet<EntityId>>,
}

impl Selection {
    /// Create a new empty selection
    pub fn new() -> Self {
        Self {
            inner: Arc::new(BTreeSet::new()),
        }
    }

    /// Create a selection from a single entity
    pub fn from_one(id: EntityId) -> Self {
        let mut set = BTreeSet::new();
        set.insert(id);
        Self {
            inner: Arc::new(set),
        }
    }

    /// Create a selection from multiple entities
    pub fn from_many(ids: impl IntoIterator<Item = EntityId>) -> Self {
        Self {
            inner: Arc::new(ids.into_iter().collect()),
        }
    }

    /// Check if the selection is empty
    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    /// Get the number of selected entities
    pub fn len(&self) -> usize {
        self.inner.len()
    }

    /// Check if an entity is selected
    pub fn contains(&self, id: &EntityId) -> bool {
        self.inner.contains(id)
    }

    /// Iterate over selected entities
    pub fn iter(&self) -> impl Iterator<Item = &EntityId> {
        self.inner.iter()
    }

    /// Add an entity to the selection
    pub fn insert(&mut self, id: EntityId) {
        let mut set = (*self.inner).clone();
        set.insert(id);
        self.inner = Arc::new(set);
    }

    /// Remove an entity from the selection
    pub fn remove(&mut self, id: &EntityId) {
        let mut set = (*self.inner).clone();
        set.remove(id);
        self.inner = Arc::new(set);
    }

    /// Toggle an entity in the selection
    pub fn toggle(&mut self, id: EntityId) {
        if self.contains(&id) {
            self.remove(&id);
        } else {
            self.insert(id);
        }
    }

    /// Clear the selection
    pub fn clear(&mut self) {
        self.inner = Arc::new(BTreeSet::new());
    }

    /// Replace the selection with a new set of entities
    pub fn set(&mut self, ids: impl IntoIterator<Item = EntityId>) {
        self.inner = Arc::new(ids.into_iter().collect());
    }

    /// Extend the selection with additional entities
    pub fn extend(&mut self, ids: impl IntoIterator<Item = EntityId>) {
        let mut set = (*self.inner).clone();
        set.extend(ids);
        self.inner = Arc::new(set);
    }

    /// Get the union of this selection with another
    pub fn union(&self, other: &Selection) -> Selection {
        let mut set = (*self.inner).clone();
        set.extend(other.inner.iter().copied());
        Selection {
            inner: Arc::new(set),
        }
    }

    /// Get the symmetric difference with another selection (XOR)
    pub fn symmetric_difference(&self, other: &Selection) -> Selection {
        let set: BTreeSet<EntityId> = self
            .inner
            .symmetric_difference(&other.inner)
            .copied()
            .collect();
        Selection {
            inner: Arc::new(set),
        }
    }
}

impl Default for Selection {
    fn default() -> Self {
        Self::new()
    }
}

impl FromIterator<EntityId> for Selection {
    fn from_iter<I: IntoIterator<Item = EntityId>>(iter: I) -> Self {
        Self::from_many(iter)
    }
}
