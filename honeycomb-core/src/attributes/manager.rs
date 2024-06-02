//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::{AttributeBind, AttributeStorage, OrbitPolicy};
use std::any::{Any, TypeId};
use std::collections::HashMap;

// ------ CONTENT

pub enum ManagerError {
    DuplicateStorage(&'static str),
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
/// collections, the internal hashmaps uses `Box<dyn Any>` as their value type. This requires us
/// to cast back the stored object (implementing `Any`) to the correct collection type. This is
/// achieved by using the associated storage type [`AttributeBind::StorageType`]. The code would
/// look like this:
///
/// ```
/// # use std::any::{Any, TypeId};
/// # use std::collections::HashMap;
/// # use honeycomb_core::{AttributeBind, AttributeStorage};
/// pub struct Manager {
///     inner: HashMap<TypeId, Box<dyn Any>>,
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
///         let probably_storage = &self.inner[TypeId::of::<A>()];
///         probably_storage
///             .downcast_ref::<<A as AttributeBind>::StorageType>()
///             .expect("E: could not downcast generic storage to specified attribute type")
///     }
/// }
/// ```
#[derive(Default)]
pub struct AttrStorageManager {
    /// Vertex attributes' storages.
    vertices: HashMap<TypeId, Box<dyn Any>>,
    /// Edges attributes' storages.
    edges: HashMap<TypeId, Box<dyn Any>>,
    /// Faces attributes' storages.
    faces: HashMap<TypeId, Box<dyn Any>>,
    /// Other storages.
    others: HashMap<TypeId, Box<dyn Any>>, // Orbit::Custom
}

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

#[allow(unused, missing_docs)]
impl AttrStorageManager {
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
            Err(ManagerError::DuplicateStorage(
                "storage of the specified type already exists",
            ))
        } else {
            Ok(())
        }
    }

    pub fn extend_storages(&mut self, length: usize) {
        // not sure if this is actually possible since we need to fetch the attribute from storages,
        // which cannot be interpreted as such without the attribute in the first place
        for storage in self.vertices.values_mut() {
            todo!()
        }
    }

    pub fn extend_storage<A: AttributeBind>(&mut self, length: usize) {
        get_storage_mut!(self, storage);
        storage.extend(length);
    }

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

    pub fn set_attribute<A: AttributeBind>(&mut self, id: A::IdentifierType, val: A) {
        get_storage_mut!(self, storage);
        storage.set(id, val);
    }

    pub fn insert_attribute<A: AttributeBind>(&mut self, id: A::IdentifierType, val: A) {
        get_storage_mut!(self, storage);
        storage.insert(id, val);
    }

    pub fn get_attribute<A: AttributeBind>(&self, id: A::IdentifierType) -> Option<A> {
        get_storage!(self, storage);
        storage.get(id)
    }

    pub fn replace_attribute<A: AttributeBind>(
        &mut self,
        id: A::IdentifierType,
        val: A,
    ) -> Option<A> {
        get_storage_mut!(self, storage);
        storage.replace(id, val)
    }

    pub fn remove_attribute<A: AttributeBind>(&mut self, id: A::IdentifierType) -> Option<A> {
        get_storage_mut!(self, storage);
        storage.remove(id)
    }
}
