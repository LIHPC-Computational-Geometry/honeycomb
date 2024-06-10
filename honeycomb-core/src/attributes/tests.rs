// ------ IMPORTS

use crate::{
    AttrCompactVec, AttrSparseVec, AttributeBind, AttributeStorage, AttributeUpdate,
    UnknownAttributeStorage,
};
use std::any::Any;

// ------ CONTENT

// --- basic structure implementation

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Temperature {
    pub val: f32,
}

impl AttributeUpdate for Temperature {
    fn merge(attr1: Self, attr2: Self) -> Self {
        Temperature {
            val: (attr1.val + attr2.val) / 2.0,
        }
    }

    fn split(attr: Self) -> (Self, Self) {
        (attr, attr)
    }

    fn merge_undefined(attr: Option<Self>) -> Self {
        attr.unwrap_or(Temperature { val: 0.0 })
    }
}

impl AttributeBind for Temperature {
    type StorageType = AttrSparseVec<Temperature>;
    type IdentifierType = crate::FaceIdentifier;
    fn binds_to<'a>() -> crate::OrbitPolicy<'a> {
        crate::OrbitPolicy::Face
    }
}

impl From<f32> for Temperature {
    fn from(val: f32) -> Self {
        Self { val }
    }
}

// --- tests

// traits

#[test]
fn attribute_update() {
    let t1 = Temperature { val: 273.0 };
    let t2 = Temperature { val: 298.0 };

    let t_new = AttributeUpdate::merge(t1, t2); // use AttributeUpdate::_
    let t_ref = Temperature { val: 285.5 };

    assert_eq!(Temperature::split(t_new), (t_ref, t_ref)); // or Temperature::_
    assert_eq!(Temperature::merge_undefined(Some(t_ref)), t_ref);
    assert_eq!(Temperature::merge_undefined(None), Temperature::from(0.0));
}

#[test]
fn attribute_bind() {
    assert_eq!(Temperature::binds_to(), crate::OrbitPolicy::Face);
    let inst: <Temperature as AttributeBind>::IdentifierType = 0;
    let ref_inst: crate::FaceIdentifier = 0;
    let prim_inst: u32 = 0;
    assert_eq!(inst.type_id(), ref_inst.type_id());
    assert_eq!(inst.type_id(), prim_inst.type_id());
}

// storages

macro_rules! generate_sparse {
    ($name: ident) => {
        let mut $name = AttrSparseVec::<Temperature>::new(10);
        $name.insert(0, Temperature::from(273.0));
        $name.insert(1, Temperature::from(275.0));
        $name.insert(2, Temperature::from(277.0));
        $name.insert(3, Temperature::from(279.0));
        $name.insert(4, Temperature::from(281.0));
        $name.insert(5, Temperature::from(283.0));
        $name.insert(6, Temperature::from(285.0));
        $name.insert(7, Temperature::from(287.0));
        $name.insert(8, Temperature::from(289.0));
        $name.insert(9, Temperature::from(291.0));
    };
}

#[test]
fn sparse_vec_n_attributes() {
    generate_sparse!(storage);
    assert_eq!(storage.n_attributes(), 10);
    let _ = storage.remove(3);
    assert_eq!(storage.n_attributes(), 9);
    // extend does not affect the number of attributes
    storage.extend(10);
    assert!(storage.get(15).is_none());
    assert_eq!(storage.n_attributes(), 9);
}

#[test]
fn sparse_vec_get_set_get() {
    generate_sparse!(storage);
    assert_eq!(storage.get(3), Some(Temperature::from(279.0)));
    storage.set(3, Temperature::from(280.0));
    assert_eq!(storage.get(3), Some(Temperature::from(280.0)));
}

#[test]
fn sparse_vec_get_replace_get() {
    generate_sparse!(storage);
    assert_eq!(storage.get(3), Some(Temperature::from(279.0)));
    storage.replace(3, Temperature::from(280.0));
    assert_eq!(storage.get(3), Some(Temperature::from(280.0)));
}

#[test]
#[should_panic(expected = "assertion failed: tmp.is_none()")]
fn sparse_vec_insert_already_existing() {
    generate_sparse!(storage);
    assert_eq!(storage.get(3), Some(Temperature::from(279.0)));
    storage.insert(3, Temperature::from(280.0)); // panic
}

#[test]
fn sparse_vec_remove() {
    generate_sparse!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
}

#[test]
fn sparse_vec_remove_remove() {
    generate_sparse!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    assert!(storage.remove(3).is_none());
}

#[test]
fn sparse_vec_remove_get() {
    generate_sparse!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    assert!(storage.get(3).is_none());
}

#[test]
fn sparse_vec_remove_set() {
    generate_sparse!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    storage.set(3, Temperature::from(280.0));
    assert!(storage.get(3).is_some());
}

#[test]
fn sparse_vec_remove_insert() {
    generate_sparse!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    storage.insert(3, Temperature::from(280.0));
    assert!(storage.get(3).is_some());
}

#[test]
#[should_panic(expected = "called `Option::unwrap()` on a `None` value")]
fn sparse_vec_replace_already_removed() {
    generate_sparse!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    storage.replace(3, Temperature::from(280.0)).unwrap(); // panic
}

macro_rules! generate_compact {
    ($name: ident) => {
        let mut $name = AttrCompactVec::<Temperature>::new(10);
        $name.insert(0, Temperature::from(273.0));
        $name.insert(1, Temperature::from(275.0));
        $name.insert(2, Temperature::from(277.0));
        $name.insert(3, Temperature::from(279.0));
        $name.insert(4, Temperature::from(281.0));
        $name.insert(5, Temperature::from(283.0));
        $name.insert(6, Temperature::from(285.0));
        $name.insert(7, Temperature::from(287.0));
        $name.insert(8, Temperature::from(289.0));
        $name.insert(9, Temperature::from(291.0));
    };
}

#[test]
fn compact_vec_n_attributes() {
    generate_compact!(storage);
    assert_eq!(storage.n_attributes(), 10);
    let _ = storage.remove(3);
    //assert_eq!(storage.n_attributes(), 10);
    assert_eq!(storage.n_attributes(), 9);
    // extend does not affect the number of attributes
    storage.extend(10);
    assert!(storage.get(15).is_none());
    assert_eq!(storage.n_attributes(), 9);
}

// FIXME: add methods to AttributeStorage to differentiate capacity / length / used length
/*
#[test]
fn compact_vec_n_used_attributes() {
    generate_compact!(storage);
    assert_eq!(storage.n_used_attributes(), 10);
    let _ = storage.remove(&3);
    assert_eq!(storage.n_used_attributes(), 9);
    // extend does not affect the number of attributes
    storage.extend(10);
    assert!(storage.get(&15).is_none());
    assert_eq!(storage.n_used_attributes(), 9);
}
 */

#[test]
fn compact_vec_extend_through_set() {
    generate_compact!(storage);
    assert_eq!(storage.n_attributes(), 10);
    // extend does not affect the number of attributes
    storage.extend(10);
    assert_eq!(storage.n_attributes(), 10);
    storage.set(10, Temperature::from(293.0));
    assert_eq!(storage.n_attributes(), 11);
    storage.set(11, Temperature::from(295.0));
    assert_eq!(storage.n_attributes(), 12);
    storage.set(12, Temperature::from(297.0));
    assert_eq!(storage.n_attributes(), 13);
    let _ = storage.remove(3);
    //assert_eq!(storage.n_attributes(), 13);
    assert_eq!(storage.n_attributes(), 12); // previously n_used_attributes
}

#[test]
fn compact_vec_get_set_get() {
    generate_compact!(storage);
    assert_eq!(storage.get(3), Some(Temperature::from(279.0)));
    storage.set(3, Temperature::from(280.0));
    assert_eq!(storage.get(3), Some(Temperature::from(280.0)));
}

#[test]
fn compact_vec_get_replace_get() {
    generate_compact!(storage);
    assert_eq!(storage.get(3), Some(Temperature::from(279.0)));
    storage.replace(3, Temperature::from(280.0));
    assert_eq!(storage.get(3), Some(Temperature::from(280.0)));
}

#[test]
#[should_panic(expected = "assertion failed: idx.is_none()")]
fn compact_vec_insert_already_existing() {
    generate_compact!(storage);
    assert_eq!(storage.get(3), Some(Temperature::from(279.0)));
    storage.insert(3, Temperature::from(280.0)); // panic
}

#[test]
fn compact_vec_remove() {
    generate_compact!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
}

#[test]
fn compact_vec_remove_remove() {
    generate_compact!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    assert!(storage.remove(3).is_none());
}

#[test]
fn compact_vec_remove_get() {
    generate_compact!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    assert!(storage.get(3).is_none());
}

#[test]
fn compact_vec_remove_set() {
    generate_compact!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    storage.set(3, Temperature::from(280.0));
    assert!(storage.get(3).is_some());
}

#[test]
fn compact_vec_remove_insert() {
    generate_compact!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    storage.insert(3, Temperature::from(280.0));
    assert!(storage.get(3).is_some());
}

#[test]
#[should_panic(expected = "assertion failed: idx.is_some()")]
fn compact_vec_replace_already_removed() {
    generate_compact!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    storage.replace(3, Temperature::from(280.0)); // panic
}
