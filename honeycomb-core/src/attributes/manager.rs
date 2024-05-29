//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use crate::AttributeBind;
use std::any::{Any, TypeId};
use std::collections::HashMap;

// ------ CONTENT

#[derive(Default)]
pub struct AttrStorageManager {
    vertices: HashMap<TypeId, Box<dyn Any>>,
    edges: HashMap<TypeId, Box<dyn Any>>,
    faces: HashMap<TypeId, Box<dyn Any>>,
    others: HashMap<TypeId, Box<dyn Any>>, // Orbit::Custom
}

impl AttrStorageManager {
    pub fn add_storage<A: AttributeBind>(&mut self) {
        todo!()
    }

    pub fn extend_storages(&mut self) {
        for storage in self.vertices.values_mut() {
            todo!()
        }
    }

    pub fn get_storage<A: AttributeBind>(&self) {
        todo!()
    }

    pub fn set_attribute<A: AttributeBind>(&self, id: A::IdentifierType, val: A) {
        todo!()
    }

    pub fn insert_attribute<A: AttributeBind>(&self, id: A::IdentifierType, val: A) {
        todo!()
    }

    pub fn get_attribute<A: AttributeBind>(&self, id: A::IdentifierType) -> A {
        todo!()
    }

    pub fn replace_attribute<A: AttributeBind>(&self, id: A::IdentifierType, val: A) -> Option<A> {
        todo!()
    }

    pub fn remove_attribute<A: AttributeBind>(&self, id: A::IdentifierType) -> Option<A> {
        todo!()
    }
}
