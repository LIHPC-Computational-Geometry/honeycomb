// ------ IMPORTS

use crate::{
    AttrCompactVec, AttrSparseVec, AttributeBind, AttributeStorage, AttributeUpdate, CMap2,
    CMapBuilder, UnknownAttributeStorage, Vertex2,
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
    type IdentifierType = crate::VertexIdentifier;
    fn binds_to<'a>() -> crate::OrbitPolicy<'a> {
        crate::OrbitPolicy::Vertex
    }
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
    let mut map: CMap2<f64> = builder.build().unwrap();
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
    assert_eq!(map.vertex(map.vertex_id(4)), Ok(Vertex2::from((2., 0.))));
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
    assert_eq!(map.vertex(map.vertex_id(2)), Ok(Vertex2::from((1., 0.))));
    assert_eq!(map.vertex(map.vertex_id(3)), Ok(Vertex2::from((1., 0.))));
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
    assert_eq!(Temperature::merge_undefined(Some(t_ref)), t_ref);
    assert_eq!(Temperature::merge_undefined(None), Temperature::from(0.0));
}

#[test]
fn attribute_bind() {
    assert_eq!(Temperature::binds_to(), crate::OrbitPolicy::Vertex);
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
    assert_eq!(storage.get(6), Some(Temperature::from(281.0)));
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
fn compact_vec_merge() {
    generate_compact!(storage);
    assert_eq!(storage.get(3), Some(Temperature::from(279.0)));
    assert_eq!(storage.get(6), Some(Temperature::from(285.0)));
    assert_eq!(storage.get(8), Some(Temperature::from(289.0)));
    storage.merge(8, 3, 6);
    assert_eq!(storage.get(3), None);
    assert_eq!(storage.get(6), None);
    assert_eq!(storage.get(8), Some(Temperature::from(282.0)));
}

#[test]
fn compact_vec_merge_undefined() {
    generate_compact!(storage);
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
    assert_eq!(storage.get(6), Some(Temperature::from(281.0)));
}

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
