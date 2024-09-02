//! attribute super structure code
//!
//! this module contains all code used to implement a manager struct, used to handle generic
//! attributes embedded in a given combinatorial map.

// ------ IMPORTS

use super::{AttributeBind, AttributeStorage, UnknownAttributeStorage};
use crate::prelude::{DartIdentifier, OrbitPolicy};
use std::{any::TypeId, collections::HashMap};

// ------ CONTENT

/// Attribute manager error enum.
#[derive(Debug)]
pub enum ManagerError {
    /// Storage of a given type already exists in the structure.
    DuplicateStorage,
}

/// Main attribute storage structure.
///
/// This structure is used to store all generic attributes that the user may add to the
/// combinatorial map he's building.
///
/// # Implementation
///
/// The structure uses hashmaps in order to store each attribute's dedicated storage. Which storage
/// is used is determined by the associated type [`AttributeBind::StorageType`].
///
/// The key type used by the map is each attribute's [`TypeId`]. This implies that all attributes
/// must have a different (unique) type, i.e. two decimal-valued attribute will need to be wrapped
/// in respective dedicated structures.
///
/// Using the [`TypeId`] as the key value for collections yields a cleaner API, where the only
/// argument passed to access methods is the ID of the cell of which they want the attribute. The
/// actual attribute type is specified by passing a generic to the method. This bypasses any issues
/// linked to literal-typed keys, such as typos, naming conventions, portability, etc.
///
/// Generics passed in access methods also have a secondary usage. To store heterogeneous
/// collections, the internal hashmaps uses `Box<dyn UnknownAttributeStorage>` as their value type.
/// Some cases require us to downcast the stored object (implementing `UnknownAttributeStorage`) to
/// the correct collection type. This is achieved by using the `downcast-rs` crate and
/// the associated storage type [`AttributeBind::StorageType`]. The code roughly looks like this:
///
/// ```
/// # use std::any::TypeId;
/// # use std::collections::HashMap;
/// # use honeycomb_core::attributes::{AttributeBind, AttributeStorage, UnknownAttributeStorage};
/// pub struct Manager {
///     inner: HashMap<TypeId, Box<dyn UnknownAttributeStorage>>,
/// }
///
/// impl Manager {
///     pub fn add_storage<A: AttributeBind + 'static>(
///         &mut self,
///         size: usize,
///     ) {
///         let typeid = TypeId::of::<A>();
///         let new_storage = <A as AttributeBind>::StorageType::new(size);
///         self.inner.insert(typeid, Box::new(new_storage));
///     }
///
///     pub fn get_storage<A: AttributeBind>(&self) -> &<A as AttributeBind>::StorageType {
///         let probably_storage = &self.inner[&TypeId::of::<A>()];
///         // downcast is possible because:
///         // - StorageType: AttributeStorage<A>
///         // - AttributeStorage<A>: UnknownAttributeStorage
///         probably_storage
///             .downcast_ref::<<A as AttributeBind>::StorageType>()
///             .expect("E: could not downcast generic storage to specified attribute type")
///     }
/// }
/// ```
#[cfg_attr(feature = "utils", derive(Clone))]
#[derive(Default)]
pub struct AttrStorageManager {
    /// Vertex attributes' storages.
    vertices: HashMap<TypeId, Box<dyn UnknownAttributeStorage>>,
    /// Edge attributes' storages.
    edges: HashMap<TypeId, Box<dyn UnknownAttributeStorage>>,
    /// Face attributes' storages.
    faces: HashMap<TypeId, Box<dyn UnknownAttributeStorage>>,
    /// Other storages.
    others: HashMap<TypeId, Box<dyn UnknownAttributeStorage>>, // Orbit::Custom
}

// --- manager-wide methods

impl AttrStorageManager {
    /// Extend the size of all storages in the manager.
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- Length by which storages should be extended.
    pub fn extend_storages(&mut self, length: usize) {
        for storage in self.vertices.values_mut() {
            storage.extend(length);
        }
        for storage in self.edges.values_mut() {
            storage.extend(length);
        }
        for storage in self.faces.values_mut() {
            storage.extend(length);
        }
        for storage in self.others.values_mut() {
            storage.extend(length);
        }
    }

    // merges

    /// Execute a merging operation on all attributes associated with a given orbit
    /// for specified cells.
    ///
    /// # Arguments
    ///
    /// - `orbit_policy: OrbitPolicy` -- Orbit associated with affected attributes.
    /// - `id_out: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in_lhs: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `id_in_rhs: DartIdentifier` -- Identifier of the other attribute value to merge.
    pub fn merge_attributes(
        &mut self,
        orbit_policy: &OrbitPolicy,
        id_out: DartIdentifier,
        id_in_lhs: DartIdentifier,
        id_in_rhs: DartIdentifier,
    ) {
        match orbit_policy {
            OrbitPolicy::Vertex => self.merge_vertex_attributes(id_out, id_in_lhs, id_in_rhs),
            OrbitPolicy::Edge => self.merge_edge_attributes(id_out, id_in_lhs, id_in_rhs),
            OrbitPolicy::Face => self.merge_face_attributes(id_out, id_in_lhs, id_in_rhs),
            OrbitPolicy::Custom(_) => {
                self.merge_other_attributes(orbit_policy, id_out, id_in_lhs, id_in_rhs);
            }
        }
    }

    /// Execute a merging operation on all attributes associated with vertices for specified cells.
    ///
    /// # Arguments
    ///
    /// - `id_out: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in_lhs: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `id_in_rhs: DartIdentifier` -- Identifier of the other attribute value to merge.
    pub fn merge_vertex_attributes(
        &mut self,
        id_out: DartIdentifier,
        id_in_lhs: DartIdentifier,
        id_in_rhs: DartIdentifier,
    ) {
        for storage in self.vertices.values_mut() {
            storage.merge(id_out, id_in_lhs, id_in_rhs);
        }
    }

    /// Execute a merging operation on all attributes associated with edges for specified cells.
    ///
    /// # Arguments
    ///
    /// - `id_out: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in_lhs: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `id_in_rhs: DartIdentifier` -- Identifier of the other attribute value to merge.
    pub fn merge_edge_attributes(
        &mut self,
        id_out: DartIdentifier,
        id_in_lhs: DartIdentifier,
        id_in_rhs: DartIdentifier,
    ) {
        for storage in self.edges.values_mut() {
            storage.merge(id_out, id_in_lhs, id_in_rhs);
        }
    }

    /// Execute a merging operation on all attributes associated with faces for specified cells.
    ///
    /// # Arguments
    ///
    /// - `id_out: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in_lhs: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `id_in_rhs: DartIdentifier` -- Identifier of the other attribute value to merge.
    pub fn merge_face_attributes(
        &mut self,
        id_out: DartIdentifier,
        id_in_lhs: DartIdentifier,
        id_in_rhs: DartIdentifier,
    ) {
        for storage in self.faces.values_mut() {
            storage.merge(id_out, id_in_lhs, id_in_rhs);
        }
    }

    /// Execute a merging operation on all attributes associated with a given orbit
    /// for specified cells.
    ///
    /// # Arguments
    ///
    /// - `orbit_policy: OrbitPolicy` -- Orbit associated with affected attributes.
    /// - `id_out: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in_lhs: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `id_in_rhs: DartIdentifier` -- Identifier of the other attribute value to merge.
    pub fn merge_other_attributes(
        &mut self,
        _orbit_policy: &OrbitPolicy,
        _id_out: DartIdentifier,
        _id_in_lhs: DartIdentifier,
        _id_in_rhs: DartIdentifier,
    ) {
        todo!("custom orbit binding is a special case that will be treated later")
    }

    // splits

    /// Execute a splitting operation on all attributes associated with a given orbit
    /// for specified cells.
    ///
    /// # Arguments
    ///
    /// - `orbit_policy: OrbitPolicy` -- Orbit associated with affected attributes.
    /// - `id_out_lhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_out_rhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in: DartIdentifier` -- Identifier of the attribute value to split.
    pub fn split_attributes(
        &mut self,
        orbit_policy: &OrbitPolicy,
        id_out_lhs: DartIdentifier,
        id_out_rhs: DartIdentifier,
        id_in: DartIdentifier,
    ) {
        match orbit_policy {
            OrbitPolicy::Vertex => self.split_vertex_attributes(id_out_lhs, id_out_rhs, id_in),
            OrbitPolicy::Edge => self.split_edge_attributes(id_out_lhs, id_out_rhs, id_in),
            OrbitPolicy::Face => self.split_face_attributes(id_out_lhs, id_out_rhs, id_in),
            OrbitPolicy::Custom(_) => {
                self.split_other_attributes(orbit_policy, id_out_lhs, id_out_rhs, id_in);
            }
        }
    }

    /// Execute a splitting operation on all attributes associated with vertices
    /// for specified cells.
    ///
    /// # Arguments
    ///
    /// - `orbit_policy: OrbitPolicy` -- Orbit associated with affected attributes.
    /// - `id_out_lhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_out_rhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in: DartIdentifier` -- Identifier of the attribute value to split.
    pub fn split_vertex_attributes(
        &mut self,
        id_out_lhs: DartIdentifier,
        id_out_rhs: DartIdentifier,
        id_in: DartIdentifier,
    ) {
        for storage in self.vertices.values_mut() {
            storage.split(id_out_lhs, id_out_rhs, id_in);
        }
    }

    /// Execute a splitting operation on all attributes associated with edges for specified cells.
    ///
    /// # Arguments
    ///
    /// - `orbit_policy: OrbitPolicy` -- Orbit associated with affected attributes.
    /// - `id_out_lhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_out_rhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in: DartIdentifier` -- Identifier of the attribute value to split.
    pub fn split_edge_attributes(
        &mut self,
        id_out_lhs: DartIdentifier,
        id_out_rhs: DartIdentifier,
        id_in: DartIdentifier,
    ) {
        for storage in self.edges.values_mut() {
            storage.split(id_out_lhs, id_out_rhs, id_in);
        }
    }

    /// Execute a splitting operation on all attributes associated with faces for specified cells.
    ///
    /// # Arguments
    ///
    /// - `orbit_policy: OrbitPolicy` -- Orbit associated with affected attributes.
    /// - `id_out_lhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_out_rhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in: DartIdentifier` -- Identifier of the attribute value to split.
    pub fn split_face_attributes(
        &mut self,
        id_out_lhs: DartIdentifier,
        id_out_rhs: DartIdentifier,
        id_in: DartIdentifier,
    ) {
        for storage in self.faces.values_mut() {
            storage.split(id_out_lhs, id_out_rhs, id_in);
        }
    }

    /// Execute a splitting operation on all attributes associated with a given orbit
    /// for specified cells.
    ///
    /// # Arguments
    ///
    /// - `orbit_policy: OrbitPolicy` -- Orbit associated with affected attributes.
    /// - `id_out_lhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_out_rhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in: DartIdentifier` -- Identifier of the attribute value to split.
    pub fn split_other_attributes(
        &mut self,
        _orbit_policy: &OrbitPolicy,
        _id_out_lhs: DartIdentifier,
        _id_out_rhs: DartIdentifier,
        _id_in: DartIdentifier,
    ) {
        todo!("custom orbit binding is a special case that will be treated later")
    }
}

// --- attribute-specific methods

macro_rules! get_storage {
    ($slf: ident, $id: ident) => {
        let probably_storage = match A::binds_to() {
            OrbitPolicy::Vertex => $slf.vertices.get(&TypeId::of::<A>()),
            OrbitPolicy::Edge => $slf.edges.get(&TypeId::of::<A>()),
            OrbitPolicy::Face => $slf.faces.get(&TypeId::of::<A>()),
            OrbitPolicy::Custom(_) => $slf.others.get(&TypeId::of::<A>()),
        };
        let $id = probably_storage
            .expect("E: could not find storage associated to the specified attribute type")
            .downcast_ref::<<A as AttributeBind>::StorageType>()
            .expect("E: could not downcast generic storage to specified attribute type");
    };
}

macro_rules! get_storage_mut {
    ($slf: ident, $id: ident) => {
        let probably_storage = match A::binds_to() {
            OrbitPolicy::Vertex => $slf.vertices.get_mut(&TypeId::of::<A>()),
            OrbitPolicy::Edge => $slf.edges.get_mut(&TypeId::of::<A>()),
            OrbitPolicy::Face => $slf.faces.get_mut(&TypeId::of::<A>()),
            OrbitPolicy::Custom(_) => $slf.others.get_mut(&TypeId::of::<A>()),
        };
        let $id = probably_storage
            .expect("E: could not find storage associated to the specified attribute type")
            .downcast_mut::<<A as AttributeBind>::StorageType>()
            .expect("E: could not downcast generic storage to specified attribute type");
    };
}

impl AttrStorageManager {
    #[allow(clippy::missing_errors_doc)]
    /// Add a new storage to the manager.
    ///
    /// For a breakdown of the principles used for implementation, refer to the *Explanation*
    /// section of the [`AttrStorageManager`] documentation entry.
    ///
    /// # Arguments
    ///
    /// - `size: usize` -- Initial size of the new storage.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind + 'static` -- Type of the attribute that will be stored.
    ///
    /// # Return / Error
    ///
    /// The function may return:
    /// - `Ok(())` if the storage was successfully added,
    /// - `Err(ManagerError::DuplicateStorage)` if there was already a storage for the specified
    ///   attribute.
    pub fn add_storage<A: AttributeBind + 'static>(
        &mut self,
        size: usize,
    ) -> Result<(), ManagerError> {
        let typeid = TypeId::of::<A>();
        let new_storage = <A as AttributeBind>::StorageType::new(size);
        if match A::binds_to() {
            OrbitPolicy::Vertex => self.vertices.insert(typeid, Box::new(new_storage)),
            OrbitPolicy::Edge => self.edges.insert(typeid, Box::new(new_storage)),
            OrbitPolicy::Face => self.faces.insert(typeid, Box::new(new_storage)),
            OrbitPolicy::Custom(_) => self.others.insert(typeid, Box::new(new_storage)),
        }
        .is_some()
        {
            Err(ManagerError::DuplicateStorage)
        } else {
            Ok(())
        }
    }

    /// Extend the size of the storage of a given attribute.
    ///
    /// # Arguments
    ///
    /// - `length: usize` -- Length by which the storage should be extended.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind` -- Attribute of which the storage should be extended.
    pub fn extend_storage<A: AttributeBind>(&mut self, length: usize) {
        get_storage_mut!(self, storage);
        storage.extend(length);
    }

    /// Get a reference to the storage of a given attribute.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Attribute stored by the fetched storage.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn UnknownAttributeStorage>` to `<A as AttributeBind>::StorageType` fails
    #[must_use = "unused getter result - please remove this method call"]
    pub fn get_storage<A: AttributeBind>(&self) -> &<A as AttributeBind>::StorageType {
        let probably_storage = match A::binds_to() {
            OrbitPolicy::Vertex => &self.vertices[&TypeId::of::<A>()],
            OrbitPolicy::Edge => &self.edges[&TypeId::of::<A>()],
            OrbitPolicy::Face => &self.faces[&TypeId::of::<A>()],
            OrbitPolicy::Custom(_) => &self.others[&TypeId::of::<A>()],
        };
        probably_storage
            .downcast_ref::<<A as AttributeBind>::StorageType>()
            .expect("E: could not downcast generic storage to specified attribute type")
    }

    /// Remove an entire attribute storage from the manager.
    ///
    /// This method is useful when implementing routines that uses attributes to run; Those can then be removed
    /// before the final result is returned.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Attribute stored by the fetched storage.
    pub fn remove_storage<A: AttributeBind>(&mut self) {
        // we could return it ?
        let _ = match A::binds_to() {
            OrbitPolicy::Vertex => &self.vertices.remove(&TypeId::of::<A>()),
            OrbitPolicy::Edge => &self.edges.remove(&TypeId::of::<A>()),
            OrbitPolicy::Face => &self.faces.remove(&TypeId::of::<A>()),
            OrbitPolicy::Custom(_) => &self.others.remove(&TypeId::of::<A>()),
        };
    }

    /// Set the value of an attribute.
    ///
    /// # Arguments
    ///
    /// - `id: A::IdentifierType` -- Cell ID to which the attribute is associated.
    /// - `val: A` -- New value of the attribute for the given ID.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Type of the attribute being set.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn UnknownAttributeStorage>` to `<A as AttributeBind>::StorageType` fails
    /// - the index lands out of bounds
    pub fn set_attribute<A: AttributeBind>(&mut self, id: A::IdentifierType, val: A) {
        get_storage_mut!(self, storage);
        storage.set(id, val);
    }

    /// Set the value of an attribute.
    ///
    /// # Arguments
    ///
    /// - `id: A::IdentifierType` -- Cell ID to which the attribute is associated.
    /// - `val: A` -- New value of the attribute for the given ID.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Type of the attribute being set.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - **there already is a value associated to the given ID for the specified attribute**
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn UnknownAttributeStorage>` to `<A as AttributeBind>::StorageType` fails
    /// - the index lands out of bounds
    pub fn insert_attribute<A: AttributeBind>(&mut self, id: A::IdentifierType, val: A) {
        get_storage_mut!(self, storage);
        storage.insert(id, val);
    }

    /// Get the value of an attribute.
    ///
    /// # Arguments
    ///
    /// - `id: A::IdentifierType` -- Cell ID to which the attribute is associated.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Type of the attribute fetched.
    ///
    /// # Return
    ///
    /// The method may return:
    /// - `Some(val: A)` if there is an attribute associated with the specified index,
    /// - `None` if there is not.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn UnknownAttributeStorage>` to `<A as AttributeBind>::StorageType` fails
    /// - the index lands out of bounds
    pub fn get_attribute<A: AttributeBind>(&self, id: A::IdentifierType) -> Option<A> {
        get_storage!(self, storage);
        storage.get(id)
    }

    /// Set the value of an attribute.
    ///
    /// # Arguments
    ///
    /// - `id: A::IdentifierType` -- ID of the cell to which the attribute is associated.
    /// - `val: A` -- New value of the attribute for the given ID.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Type of the attribute being set.
    ///
    /// # Return
    ///
    /// The method should return:
    /// - `Some(val_old: A)` if there was an attribute associated with the specified index,
    /// - `None` if there was not.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn UnknownAttributeStorage>` to `<A as AttributeBind>::StorageType` fails
    /// - the index lands out of bounds
    pub fn replace_attribute<A: AttributeBind>(
        &mut self,
        id: A::IdentifierType,
        val: A,
    ) -> Option<A> {
        get_storage_mut!(self, storage);
        storage.replace(id, val)
    }

    /// Remove the an item from an attribute storage.
    ///
    /// # Arguments
    ///
    /// - `id: A::IdentifierType` -- Cell ID to which the attribute is associated.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind` -- Type of the attribute fetched.
    ///
    /// # Return
    ///
    /// The method may return:
    /// - `Some(val: A)` if was is an attribute associated with the specified index,
    /// - `None` if there was not.
    ///
    /// # Panics
    ///
    /// This method may panic if:
    /// - there's no storage associated with the specified attribute
    /// - downcasting `Box<dyn UnknownAttributeStorage>` to `<A as AttributeBind>::StorageType` fails
    /// - the index lands out of bounds
    pub fn remove_attribute<A: AttributeBind>(&mut self, id: A::IdentifierType) -> Option<A> {
        get_storage_mut!(self, storage);
        storage.remove(id)
    }

    /// Merge given attribute values.
    ///
    /// # Arguments
    ///
    /// - `id_out: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in_lhs: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `id_in_rhs: DartIdentifier` -- Identifier of the other attribute value to merge.
    pub fn merge_attribute<A: AttributeBind>(
        &mut self,
        id_out: DartIdentifier,
        id_in_lhs: DartIdentifier,
        id_in_rhs: DartIdentifier,
    ) {
        get_storage_mut!(self, storage);
        storage.merge(id_out, id_in_lhs, id_in_rhs);
    }

    /// Split given attribute value.
    ///
    /// # Arguments
    ///
    /// - `id_out_lhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_out_rhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in: DartIdentifier` -- Identifier of the attribute value to split.
    pub fn split_attribute<A: AttributeBind>(
        &mut self,
        id_out_lhs: DartIdentifier,
        id_out_rhs: DartIdentifier,
        id_in: DartIdentifier,
    ) {
        get_storage_mut!(self, storage);
        storage.split(id_out_lhs, id_out_rhs, id_in);
    }
}
