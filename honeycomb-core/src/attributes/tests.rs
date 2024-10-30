// ------ IMPORTS

use super::{
    /*AttrCompactVec,*/ AttrSparseVec, AttrStorageManager, AttributeBind, AttributeStorage,
    AttributeUpdate, UnknownAttributeStorage,
};
use crate::prelude::{CMap2, CMapBuilder, FaceIdentifier, OrbitPolicy, Vertex2, VertexIdentifier};
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

    fn merge_incomplete(attr: Self) -> Self {
        Temperature::from(attr.val / 2.0)
    }

    fn merge_from_none() -> Option<Self> {
        Some(Temperature::from(0.0))
    }
}

impl AttributeBind for Temperature {
    type StorageType = AttrSparseVec<Temperature>;
    type IdentifierType = VertexIdentifier;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}

impl From<f32> for Temperature {
    fn from(val: f32) -> Self {
        Self { val }
    }
}

// --- usual workflow test

#[test]
fn temperature_map() {
    // build the map
    let builder = CMapBuilder::default()
        .n_darts(6)
        .add_attribute::<Temperature>();
    let map: CMap2<f64> = builder.build().unwrap();
    map.two_link(1, 2);
    map.two_link(3, 4);
    map.two_link(5, 6);
    map.one_link(1, 3);
    map.insert_vertex(1, (0.0, 0.0));
    map.insert_vertex(2, (1.0, 0.0));
    map.insert_vertex(4, (1.5, 0.0));
    map.insert_vertex(5, (2.5, 0.0));
    map.insert_vertex(6, (3.0, 0.0));
    map.set_attribute::<Temperature>(1, Temperature::from(273.));
    map.set_attribute::<Temperature>(2, Temperature::from(275.));
    map.set_attribute::<Temperature>(4, Temperature::from(277.));
    map.set_attribute::<Temperature>(5, Temperature::from(273.));
    map.set_attribute::<Temperature>(6, Temperature::from(273.));
    // test the map
    assert_eq!(
        map.get_attribute::<Temperature>(map.vertex_id(4)),
        Some(Temperature::from(277.))
    );
    assert_eq!(
        map.get_attribute::<Temperature>(map.vertex_id(5)),
        Some(Temperature::from(273.))
    );
    // sew one segment
    map.one_sew(3, 5);
    assert_eq!(map.vertex_id(4), map.vertex_id(5));
    assert_eq!(
        map.get_attribute::<Temperature>(map.vertex_id(4)),
        Some(Temperature::from(275.))
    );
    assert_eq!(map.vertex(map.vertex_id(4)), Some(Vertex2::from((2., 0.))));
    // unsew another
    map.one_unsew(1);
    assert_ne!(map.vertex_id(2), map.vertex_id(3));
    assert_eq!(
        map.get_attribute::<Temperature>(map.vertex_id(2)),
        Some(Temperature::from(275.))
    );
    assert_eq!(
        map.get_attribute::<Temperature>(map.vertex_id(3)),
        Some(Temperature::from(275.))
    );
    assert_eq!(map.vertex(map.vertex_id(2)), Some(Vertex2::from((1., 0.))));
    assert_eq!(map.vertex(map.vertex_id(3)), Some(Vertex2::from((1., 0.))));
}

// --- unit tests

// traits

#[test]
fn attribute_update() {
    let t1 = Temperature { val: 273.0 };
    let t2 = Temperature { val: 298.0 };

    let t_new = AttributeUpdate::merge(t1, t2); // use AttributeUpdate::_
    let t_ref = Temperature { val: 285.5 };

    assert_eq!(Temperature::split(t_new), (t_ref, t_ref)); // or Temperature::_
    assert_eq!(
        Temperature::merge_incomplete(t_ref),
        Temperature::from(t_ref.val / 2.0)
    );
    assert_eq!(Temperature::merge_from_none(), Some(Temperature::from(0.0)));
}

#[test]
fn attribute_bind() {
    assert_eq!(Temperature::BIND_POLICY, OrbitPolicy::Vertex);
    let inst: <Temperature as AttributeBind>::IdentifierType = 0;
    let ref_inst: FaceIdentifier = 0;
    let prim_inst: u32 = 0;
    assert_eq!(inst.type_id(), ref_inst.type_id());
    assert_eq!(inst.type_id(), prim_inst.type_id());
}

// storages

macro_rules! generate_sparse {
    ($name: ident) => {
        #[allow(unused_mut)]
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
fn sparse_vec_merge() {
    generate_sparse!(storage);
    assert_eq!(storage.get(3), Some(Temperature::from(279.0)));
    assert_eq!(storage.get(6), Some(Temperature::from(285.0)));
    assert_eq!(storage.get(8), Some(Temperature::from(289.0)));
    storage.merge(8, 3, 6);
    assert_eq!(storage.get(3), None);
    assert_eq!(storage.get(6), None);
    assert_eq!(storage.get(8), Some(Temperature::from(282.0)));
}

#[test]
fn sparse_vec_merge_undefined() {
    generate_sparse!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    assert_eq!(storage.remove(6), Some(Temperature::from(285.0)));
    assert_eq!(storage.remove(8), Some(Temperature::from(289.0)));
    // merge from two undefined value
    storage.merge(8, 3, 6);
    assert_eq!(storage.get(3), None);
    assert_eq!(storage.get(6), None);
    assert_eq!(storage.get(8), Some(Temperature::from(0.0)));
    // merge from one undefined value
    assert_eq!(storage.get(4), Some(Temperature::from(281.0)));
    storage.merge(6, 3, 4);
    assert_eq!(storage.get(3), None);
    assert_eq!(storage.get(4), None);
    assert_eq!(storage.get(6), Some(Temperature::from(281.0 / 2.0)));
}

#[test]
fn sparse_vec_split() {
    generate_sparse!(storage);
    assert_eq!(storage.remove(3), Some(Temperature::from(279.0)));
    assert_eq!(storage.remove(6), Some(Temperature::from(285.0)));
    assert_eq!(storage.get(8), Some(Temperature::from(289.0)));
    storage.split(3, 6, 8);
    assert_eq!(storage.get(3), Some(Temperature::from(289.0)));
    assert_eq!(storage.get(6), Some(Temperature::from(289.0)));
    assert_eq!(storage.get(8), None);
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

// storage manager

macro_rules! generate_manager {
    ($name: ident) => {
        let mut $name = AttrStorageManager::default();
        $name.add_storage::<Temperature>(10);
        $name.insert_attribute(0, Temperature::from(273.0));
        $name.insert_attribute(1, Temperature::from(275.0));
        $name.insert_attribute(2, Temperature::from(277.0));
        $name.insert_attribute(3, Temperature::from(279.0));
        $name.insert_attribute(4, Temperature::from(281.0));
        $name.insert_attribute(5, Temperature::from(283.0));
        $name.insert_attribute(6, Temperature::from(285.0));
        $name.insert_attribute(7, Temperature::from(287.0));
        $name.insert_attribute(8, Temperature::from(289.0));
        $name.insert_attribute(9, Temperature::from(291.0));
    };
}

#[allow(clippy::cast_precision_loss)]
#[test]
fn manager_extend() {
    generate_manager!(manager);
    assert_eq!(
        manager.get_storage::<Temperature>().unwrap().n_attributes(),
        10
    );
    manager.extend_storage::<Temperature>(10);
    assert_eq!(
        manager.get_storage::<Temperature>().unwrap().n_attributes(),
        10
    );
    (10..20)
        .for_each(|id| manager.insert_attribute(id, Temperature::from(273.0 + 2.0 * id as f32)));
    assert_eq!(
        manager.get_storage::<Temperature>().unwrap().n_attributes(),
        20
    );
}

#[test]
#[should_panic(expected = "index out of bounds: the len is 10 but the index is 15")]
fn manager_set_oob() {
    generate_manager!(manager);
    assert_eq!(
        manager.get_storage::<Temperature>().unwrap().n_attributes(),
        10
    );
    manager.insert_attribute(15, Temperature::from(0.0)); // panic
}

#[test]
fn manager_get_set_get() {
    generate_manager!(manager);
    assert_eq!(manager.get_attribute(3), Some(Temperature::from(279.0)));
    manager.set_attribute(3, Temperature::from(280.0));
    assert_eq!(manager.get_attribute(3), Some(Temperature::from(280.0)));
}

#[test]
fn manager_vec_get_replace_get() {
    generate_manager!(manager);
    assert_eq!(manager.get_attribute(3), Some(Temperature::from(279.0)));
    manager.replace_attribute(3, Temperature::from(280.0));
    assert_eq!(manager.get_attribute(3), Some(Temperature::from(280.0)));
}

// expect tmp.is_none since Temperate::StorageType is AttrSparseVec
#[test]
#[should_panic(expected = "assertion failed: tmp.is_none()")]
fn manager_vec_insert_already_existing() {
    generate_manager!(manager);
    assert_eq!(manager.get_attribute(3), Some(Temperature::from(279.0)));
    manager.insert_attribute(3, Temperature::from(280.0)); // panic
}

#[test]
fn manager_vec_remove_remove() {
    generate_manager!(manager);
    assert_eq!(manager.remove_attribute(3), Some(Temperature::from(279.0)));
    assert!(manager.remove_attribute::<Temperature>(3).is_none());
}

#[test]
fn manager_vec_remove_get() {
    generate_manager!(manager);
    assert_eq!(manager.remove_attribute(3), Some(Temperature::from(279.0)));
    assert!(manager.get_attribute::<Temperature>(3).is_none());
}

#[test]
fn manager_vec_remove_set() {
    generate_manager!(manager);
    assert_eq!(manager.remove_attribute(3), Some(Temperature::from(279.0)));
    manager.set_attribute(3, Temperature::from(280.0));
    assert!(manager.get_attribute::<Temperature>(3).is_some());
}

#[test]
fn manager_vec_remove_insert() {
    generate_manager!(manager);
    assert_eq!(manager.remove_attribute(3), Some(Temperature::from(279.0)));
    manager.insert_attribute(3, Temperature::from(280.0));
    assert!(manager.get_attribute::<Temperature>(3).is_some());
}

#[test]
fn manager_vec_replace_already_removed() {
    generate_manager!(manager);
    assert_eq!(manager.remove_attribute(3), Some(Temperature::from(279.0)));
    assert!(manager
        .replace_attribute(3, Temperature::from(280.0))
        .is_none());
}

#[test]
fn manager_merge_attribute() {
    generate_manager!(manager);
    assert_eq!(manager.get_attribute(3), Some(Temperature::from(279.0)));
    assert_eq!(manager.get_attribute(6), Some(Temperature::from(285.0)));
    assert_eq!(manager.get_attribute(8), Some(Temperature::from(289.0)));
    manager.merge_attribute::<Temperature>(8, 3, 6);
    assert_eq!(manager.get_attribute::<Temperature>(3), None);
    assert_eq!(manager.get_attribute::<Temperature>(6), None);
    assert_eq!(manager.get_attribute(8), Some(Temperature::from(282.0)));
}

#[test]
fn manager_merge_undefined_attribute() {
    generate_manager!(manager);
    assert_eq!(manager.remove_attribute(3), Some(Temperature::from(279.0)));
    assert_eq!(manager.remove_attribute(6), Some(Temperature::from(285.0)));
    assert_eq!(manager.remove_attribute(8), Some(Temperature::from(289.0)));
    // merge from two undefined value
    manager.merge_attribute::<Temperature>(8, 3, 6);
    assert_eq!(manager.get_attribute::<Temperature>(3), None);
    assert_eq!(manager.get_attribute::<Temperature>(6), None);
    assert_eq!(manager.get_attribute(8), Some(Temperature::from(0.0)));
    // merge from one undefined value
    assert_eq!(manager.get_attribute(4), Some(Temperature::from(281.0)));
    manager.merge_attribute::<Temperature>(6, 3, 4);
    assert_eq!(manager.get_attribute::<Temperature>(3), None);
    assert_eq!(manager.get_attribute::<Temperature>(4), None);
    assert_eq!(
        manager.get_attribute(6),
        Some(Temperature::from(281.0 / 2.0))
    );
}

#[test]
fn manager_split_attribute() {
    generate_manager!(manager);
    assert_eq!(manager.remove_attribute(3), Some(Temperature::from(279.0)));
    assert_eq!(manager.remove_attribute(6), Some(Temperature::from(285.0)));
    assert_eq!(manager.get_attribute(8), Some(Temperature::from(289.0)));
    manager.split_attribute::<Temperature>(3, 6, 8);
    assert_eq!(manager.get_attribute(3), Some(Temperature::from(289.0)));
    assert_eq!(manager.get_attribute(6), Some(Temperature::from(289.0)));
    assert_eq!(manager.get_attribute::<Temperature>(8), None);
}
