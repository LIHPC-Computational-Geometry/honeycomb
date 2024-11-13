//! attribute super structure code
//!
//! this module contains all code used to implement a manager struct, used to handle generic
//! attributes embedded in a given combinatorial map.

// ------ IMPORTS

use stm::{StmResult, Transaction};

use super::{AttributeBind, AttributeStorage, AttributeUpdate, UnknownAttributeStorage};
use crate::{
    cmap::CMapResult,
    prelude::{DartIdType, OrbitPolicy},
};
use std::{any::TypeId, collections::HashMap};

// ------ CONTENT

// convenience macros

macro_rules! get_storage {
    ($slf: ident, $id: ident) => {
        let probably_storage = match A::BIND_POLICY {
            OrbitPolicy::Vertex => $slf.vertices.get(&TypeId::of::<A>()),
            OrbitPolicy::Edge => $slf.edges.get(&TypeId::of::<A>()),
            OrbitPolicy::Face => $slf.faces.get(&TypeId::of::<A>()),
            OrbitPolicy::Custom(_) => $slf.others.get(&TypeId::of::<A>()),
        };
        let $id = probably_storage
            .map(|m| m.downcast_ref::<<A as AttributeBind>::StorageType>())
            .flatten();
    };
}

macro_rules! get_storage_mut {
    ($slf: ident, $id: ident) => {
        let probably_storage = match A::BIND_POLICY {
            OrbitPolicy::Vertex => $slf.vertices.get_mut(&TypeId::of::<A>()),
            OrbitPolicy::Edge => $slf.edges.get_mut(&TypeId::of::<A>()),
            OrbitPolicy::Face => $slf.faces.get_mut(&TypeId::of::<A>()),
            OrbitPolicy::Custom(_) => $slf.others.get_mut(&TypeId::of::<A>()),
        };
        let $id = probably_storage
            .map(|m| m.downcast_mut::<<A as AttributeBind>::StorageType>())
            .flatten();
    };
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
/// must have a different (unique) type. For example, two decimal-valued attribute will need to be
/// wrapped in different dedicated structures.
///
/// Using the [`TypeId`] as the key value for collections yields a cleaner API, where the only
/// argument passed to access methods is the ID of the cell of which they want the attribute. The
/// actual attribute type is specified by passing a generic to the method. This bypasses any issues
/// linked to literal-typed keys, such as typos, naming conventions, portability, etc.
///
/// Generics passed in access methods also have a secondary usage. To store heterogeneous
/// collections, the internal hashmaps uses `Box<dyn UnknownAttributeStorage>` as their value type.
/// Some operations require us to downcast the stored object (implementing
/// `UnknownAttributeStorage`) to the correct collection type. This is achieved by using the
/// `downcast-rs` crate and the associated storage type [`AttributeBind::StorageType`]. What
/// follows is a simplified version of that code:
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

unsafe impl Send for AttrStorageManager {}
unsafe impl Sync for AttrStorageManager {}

/// **General methods**
impl AttrStorageManager {
    // attribute-agnostic

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

    // attribute-specific

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
    /// # Panics
    ///
    /// This function will panic if there is already a storage of attribute `A` in the manager.
    pub fn add_storage<A: AttributeBind + 'static>(&mut self, size: usize) {
        let typeid = TypeId::of::<A>();
        let new_storage = <A as AttributeBind>::StorageType::new(size);
        if match A::BIND_POLICY {
            OrbitPolicy::Vertex => self.vertices.insert(typeid, Box::new(new_storage)),
            OrbitPolicy::Edge => self.edges.insert(typeid, Box::new(new_storage)),
            OrbitPolicy::Face => self.faces.insert(typeid, Box::new(new_storage)),
            OrbitPolicy::Custom(_) => self.others.insert(typeid, Box::new(new_storage)),
        }
        .is_some()
        {
            eprintln!(
                "W: Storage of attribute `{}` already exists in the attribute storage manager",
                std::any::type_name::<A>()
            );
            eprintln!("   Continuing...");
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
        if let Some(st) = storage {
            st.extend(length);
        } else {
            eprintln!(
                "W: could not extend storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
        }
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
    pub fn get_storage<A: AttributeBind>(&self) -> Option<&<A as AttributeBind>::StorageType> {
        let probably_storage = match A::BIND_POLICY {
            OrbitPolicy::Vertex => &self.vertices[&TypeId::of::<A>()],
            OrbitPolicy::Edge => &self.edges[&TypeId::of::<A>()],
            OrbitPolicy::Face => &self.faces[&TypeId::of::<A>()],
            OrbitPolicy::Custom(_) => &self.others[&TypeId::of::<A>()],
        };
        probably_storage.downcast_ref::<<A as AttributeBind>::StorageType>()
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
        let _ = match A::BIND_POLICY {
            OrbitPolicy::Vertex => &self.vertices.remove(&TypeId::of::<A>()),
            OrbitPolicy::Edge => &self.edges.remove(&TypeId::of::<A>()),
            OrbitPolicy::Face => &self.faces.remove(&TypeId::of::<A>()),
            OrbitPolicy::Custom(_) => &self.others.remove(&TypeId::of::<A>()),
        };
    }
}

/// Merge variants.
impl AttrStorageManager {
    // attribute-agnostic force

    pub fn force_merge_attributes(
        &self,
        orbit_policy: &OrbitPolicy,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) {
        match orbit_policy {
            OrbitPolicy::Vertex => self.force_merge_vertex_attributes(id_out, id_in_lhs, id_in_rhs),
            OrbitPolicy::Edge => self.force_merge_edge_attributes(id_out, id_in_lhs, id_in_rhs),
            OrbitPolicy::Face => self.force_merge_face_attributes(id_out, id_in_lhs, id_in_rhs),
            OrbitPolicy::Custom(_) => unimplemented!(),
        }
    }

    pub fn force_merge_vertex_attributes(
        &self,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) {
        for storage in self.vertices.values() {
            storage.force_merge(id_out, id_in_lhs, id_in_rhs);
        }
    }

    pub fn force_merge_edge_attributes(
        &self,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) {
        for storage in self.edges.values() {
            storage.force_merge(id_out, id_in_lhs, id_in_rhs);
        }
    }

    pub fn force_merge_face_attributes(
        &self,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) {
        for storage in self.faces.values() {
            storage.force_merge(id_out, id_in_lhs, id_in_rhs);
        }
    }

    // attribute-agnostic regular

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
        &self,
        trans: &mut Transaction,
        orbit_policy: &OrbitPolicy,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> StmResult<()> {
        match orbit_policy {
            OrbitPolicy::Vertex => {
                self.merge_vertex_attributes(trans, id_out, id_in_lhs, id_in_rhs)
            }
            OrbitPolicy::Edge => self.merge_edge_attributes(trans, id_out, id_in_lhs, id_in_rhs),
            OrbitPolicy::Face => self.merge_face_attributes(trans, id_out, id_in_lhs, id_in_rhs),
            OrbitPolicy::Custom(_) => unimplemented!(),
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
        &self,
        trans: &mut Transaction,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> StmResult<()> {
        for storage in self.vertices.values() {
            storage.merge(trans, id_out, id_in_lhs, id_in_rhs)?;
        }
        Ok(())
    }

    /// Execute a merging operation on all attributes associated with edges for specified cells.
    ///
    /// # Arguments
    ///
    /// - `id_out: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in_lhs: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `id_in_rhs: DartIdentifier` -- Identifier of the other attribute value to merge.
    pub fn merge_edge_attributes(
        &self,
        trans: &mut Transaction,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> StmResult<()> {
        for storage in self.edges.values() {
            storage.merge(trans, id_out, id_in_lhs, id_in_rhs)?;
        }
        Ok(())
    }

    /// Execute a merging operation on all attributes associated with faces for specified cells.
    ///
    /// # Arguments
    ///
    /// - `id_out: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in_lhs: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `id_in_rhs: DartIdentifier` -- Identifier of the other attribute value to merge.
    pub fn merge_face_attributes(
        &self,
        trans: &mut Transaction,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> StmResult<()> {
        for storage in self.faces.values() {
            storage.merge(trans, id_out, id_in_lhs, id_in_rhs)?;
        }
        Ok(())
    }

    // attribute-agnostic try

    pub fn try_merge_attributes(
        &self,
        trans: &mut Transaction,
        orbit_policy: &OrbitPolicy,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> CMapResult<()> {
        match orbit_policy {
            OrbitPolicy::Vertex => {
                self.try_merge_vertex_attributes(trans, id_out, id_in_lhs, id_in_rhs)
            }
            OrbitPolicy::Edge => {
                self.try_merge_edge_attributes(trans, id_out, id_in_lhs, id_in_rhs)
            }
            OrbitPolicy::Face => {
                self.try_merge_face_attributes(trans, id_out, id_in_lhs, id_in_rhs)
            }
            OrbitPolicy::Custom(_) => unimplemented!(),
        }
    }

    pub fn try_merge_vertex_attributes(
        &self,
        trans: &mut Transaction,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> CMapResult<()> {
        for storage in self.vertices.values() {
            storage.try_merge(trans, id_out, id_in_lhs, id_in_rhs)?;
        }
        Ok(())
    }

    pub fn try_merge_edge_attributes(
        &self,
        trans: &mut Transaction,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> CMapResult<()> {
        for storage in self.edges.values() {
            storage.try_merge(trans, id_out, id_in_lhs, id_in_rhs)?;
        }
        Ok(())
    }

    pub fn try_merge_face_attributes(
        &self,
        trans: &mut Transaction,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> CMapResult<()> {
        for storage in self.faces.values() {
            storage.try_merge(trans, id_out, id_in_lhs, id_in_rhs)?;
        }
        Ok(())
    }

    // attribute-specific

    pub fn force_merge_attribute<A: AttributeBind>(
        &self,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.force_merge(id_out, id_in_lhs, id_in_rhs);
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
        }
    }

    /// Merge given attribute values.
    ///
    /// # Arguments
    ///
    /// - `id_out: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in_lhs: DartIdentifier` -- Identifier of one attribute value to merge.
    /// - `id_in_rhs: DartIdentifier` -- Identifier of the other attribute value to merge.
    pub fn merge_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> StmResult<()> {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.merge(trans, id_out, id_in_lhs, id_in_rhs)
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
            Ok(())
        }
    }

    pub fn try_merge_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> CMapResult<()> {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.try_merge(trans, id_out, id_in_lhs, id_in_rhs)
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
            Ok(())
        }
    }
}

/// Split variants.
impl AttrStorageManager {
    // attribute-agnostic force

    pub fn force_split_attributes(
        &self,
        orbit_policy: &OrbitPolicy,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) {
        match orbit_policy {
            OrbitPolicy::Vertex => {
                self.force_split_vertex_attributes(id_out_lhs, id_out_rhs, id_in)
            }
            OrbitPolicy::Edge => self.force_split_edge_attributes(id_out_lhs, id_out_rhs, id_in),
            OrbitPolicy::Face => self.force_split_face_attributes(id_out_lhs, id_out_rhs, id_in),
            OrbitPolicy::Custom(_) => unimplemented!(),
        }
    }

    pub fn force_split_vertex_attributes(
        &self,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) {
        for storage in self.vertices.values() {
            storage.force_split(id_out_lhs, id_out_rhs, id_in);
        }
    }

    pub fn force_split_edge_attributes(
        &self,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) {
        for storage in self.edges.values() {
            storage.force_split(id_out_lhs, id_out_rhs, id_in);
        }
    }

    pub fn force_split_face_attributes(
        &self,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) {
        for storage in self.faces.values() {
            storage.force_split(id_out_lhs, id_out_rhs, id_in);
        }
    }

    // attribute-agnostic regular

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
        &self,
        trans: &mut Transaction,
        orbit_policy: &OrbitPolicy,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> StmResult<()> {
        match orbit_policy {
            OrbitPolicy::Vertex => {
                self.split_vertex_attributes(trans, id_out_lhs, id_out_rhs, id_in)
            }
            OrbitPolicy::Edge => self.split_edge_attributes(trans, id_out_lhs, id_out_rhs, id_in),
            OrbitPolicy::Face => self.split_face_attributes(trans, id_out_lhs, id_out_rhs, id_in),
            OrbitPolicy::Custom(_) => unimplemented!(),
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
        &self,
        trans: &mut Transaction,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> StmResult<()> {
        for storage in self.vertices.values() {
            storage.split(trans, id_out_lhs, id_out_rhs, id_in)?;
        }
        Ok(())
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
        &self,
        trans: &mut Transaction,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> StmResult<()> {
        for storage in self.edges.values() {
            storage.split(trans, id_out_lhs, id_out_rhs, id_in)?;
        }
        Ok(())
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
        &self,
        trans: &mut Transaction,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> StmResult<()> {
        for storage in self.faces.values() {
            storage.split(trans, id_out_lhs, id_out_rhs, id_in)?;
        }
        Ok(())
    }

    // attribute-agnostic try

    pub fn try_split_attributes(
        &self,
        trans: &mut Transaction,
        orbit_policy: &OrbitPolicy,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> CMapResult<()> {
        match orbit_policy {
            OrbitPolicy::Vertex => {
                self.try_split_vertex_attributes(trans, id_out_lhs, id_out_rhs, id_in)
            }
            OrbitPolicy::Edge => {
                self.try_split_edge_attributes(trans, id_out_lhs, id_out_rhs, id_in)
            }
            OrbitPolicy::Face => {
                self.try_split_face_attributes(trans, id_out_lhs, id_out_rhs, id_in)
            }
            OrbitPolicy::Custom(_) => unimplemented!(),
        }
    }

    pub fn try_split_vertex_attributes(
        &self,
        trans: &mut Transaction,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> CMapResult<()> {
        for storage in self.vertices.values() {
            storage.try_split(trans, id_out_lhs, id_out_rhs, id_in)?;
        }
        Ok(())
    }

    pub fn try_split_edge_attributes(
        &self,
        trans: &mut Transaction,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> CMapResult<()> {
        for storage in self.edges.values() {
            storage.try_split(trans, id_out_lhs, id_out_rhs, id_in)?;
        }
        Ok(())
    }

    pub fn try_split_face_attributes(
        &self,
        trans: &mut Transaction,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> CMapResult<()> {
        for storage in self.faces.values() {
            storage.try_split(trans, id_out_lhs, id_out_rhs, id_in)?;
        }
        Ok(())
    }

    // attribute-specific

    pub fn force_split_attribute<A: AttributeBind>(
        &self,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.force_split(id_out_lhs, id_out_rhs, id_in);
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
        }
    }

    /// Split given attribute value.
    ///
    /// # Arguments
    ///
    /// - `id_out_lhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_out_rhs: DartIdentifier` -- Identifier to write the result to.
    /// - `id_in: DartIdentifier` -- Identifier of the attribute value to split.
    pub fn split_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> StmResult<()> {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.split(trans, id_out_lhs, id_out_rhs, id_in)
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
            Ok(())
        }
    }

    pub fn try_split_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> CMapResult<()> {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.try_split(trans, id_out_lhs, id_out_rhs, id_in)
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
            Ok(())
        }
    }
}

/// **Attribute read & write methods**
impl AttrStorageManager {
    pub fn force_write_attribute<A: AttributeBind>(
        &self,
        id: A::IdentifierType,
        val: A,
    ) -> Option<A> {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.force_write(id, val)
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
            None
        }
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
    pub fn write_attribute<A: AttributeBind>(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
        val: A,
    ) -> StmResult<Option<A>> {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.write(trans, id, val)
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
            Ok(None)
        }
    }

    pub fn force_read_attribute<A: AttributeBind>(&self, id: A::IdentifierType) -> Option<A> {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.force_read(id)
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
            None
        }
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
    pub fn read_attribute<A: AttributeBind>(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
    ) -> StmResult<Option<A>> {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.read(trans, id)
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
            Ok(None)
        }
    }

    pub fn force_remove_attribute<A: AttributeBind>(&self, id: A::IdentifierType) -> Option<A> {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.force_remove(id)
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
            None
        }
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
    pub(crate) fn remove_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
    ) -> StmResult<Option<A>> {
        get_storage!(self, storage);
        if let Some(st) = storage {
            st.remove(trans, id)
        } else {
            eprintln!(
                "W: could not update storage of attribute {} - storage not found",
                std::any::type_name::<A>()
            );
            Ok(None)
        }
    }
}
