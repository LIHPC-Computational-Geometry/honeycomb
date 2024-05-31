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

#[derive(Default)]
pub struct AttrStorageManager {
    vertices: HashMap<TypeId, Box<dyn Any>>,
    edges: HashMap<TypeId, Box<dyn Any>>,
    faces: HashMap<TypeId, Box<dyn Any>>,
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
