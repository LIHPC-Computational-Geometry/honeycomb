//! Module short description
//!
//! Should you interact with this module directly?
//!
//! Content description if needed

// ------ IMPORTS

use std::any::{Any, TypeId};
use std::collections::HashMap;

// ------ CONTENT

pub struct AttrStorageManager {
    vertices: HashMap<TypeId, Box<dyn Any>>,
    edges: HashMap<TypeId, Box<dyn Any>>,
    faces: HashMap<TypeId, Box<dyn Any>>,
    others: HashMap<TypeId, Box<dyn Any>>, // Orbit::Custom
}
