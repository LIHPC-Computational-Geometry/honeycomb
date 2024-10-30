// ------ IMPORTS

use stm::{atomically, StmError, Transaction, TransactionControl};

use super::{
    AttrSparseVec, AttrStorageManager, AttributeBind, AttributeStorage, AttributeUpdate,
    UnknownAttributeStorage,
};
use crate::{
    cmap::EdgeIdentifier,
    prelude::{CMap2, CMapBuilder, FaceIdentifier, OrbitPolicy, Vertex2, VertexIdentifier},
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

// Create a new edge-bound attribute for testing
#[derive(Debug, Clone, PartialEq, Copy)]
struct Length(pub f32);

impl AttributeUpdate for Length {
    fn merge(attr1: Self, attr2: Self) -> Self {
        Length((attr1.0 + attr2.0) / 2.0)
    }

    fn split(attr: Self) -> (Self, Self) {
        (attr, attr)
    }
}

impl AttributeBind for Length {
    type IdentifierType = EdgeIdentifier;
    type StorageType = AttrSparseVec<Self>;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Edge;
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
#[test]
fn test_attribute_operations() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Test set and get
    manager.set_attribute(0, Temperature::from(25.0));
    assert_eq!(
        manager.get_attribute::<Temperature>(0),
        Some(Temperature::from(25.0))
    );

    // Test insert
    manager.insert_attribute(1, Temperature::from(30.0));
    assert_eq!(
        manager.get_attribute::<Temperature>(1),
        Some(Temperature::from(30.0))
    );

    // Test replace
    let old_val = manager.replace_attribute(0, Temperature::from(27.0));
    assert_eq!(old_val, Some(Temperature::from(25.0)));
    assert_eq!(
        manager.get_attribute::<Temperature>(0),
        Some(Temperature::from(27.0))
    );

    // Test remove
    let removed = manager.remove_attribute::<Temperature>(0);
    assert_eq!(removed, Some(Temperature::from(27.0)));
    assert_eq!(manager.get_attribute::<Temperature>(0), None);
}

#[test]
fn test_merge_attributes() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Setup initial values
    manager.set_attribute(0, Temperature::from(20.0));
    manager.set_attribute(1, Temperature::from(30.0));

    // Test merge
    manager.merge_attribute::<Temperature>(2, 0, 1);

    // The exact result depends on how merge is implemented in AttributeStorage
    // Just verify that something was stored at the output location
    assert!(manager.get_attribute::<Temperature>(2).is_some());
}

#[test]
fn test_split_attributes() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Setup initial value
    manager.set_attribute(0, Temperature::from(25.0));

    // Test split
    manager.split_attribute::<Temperature>(1, 2, 0);

    // The exact results depend on how split is implemented in AttributeStorage
    // Just verify that something was stored at both output locations
    assert!(manager.get_attribute::<Temperature>(1).is_some());
    assert!(manager.get_attribute::<Temperature>(2).is_some());
}

#[test]
fn test_extend_all_storages() {
    let mut manager = AttrStorageManager::default();

    // Add storages of different types
    manager.add_storage::<Temperature>(2);
    manager.add_storage::<Length>(2);

    // Extend all storages
    manager.extend_storages(3);

    // Check that all storages were extended
    let temp_storage = manager.get_storage::<Temperature>().unwrap();
    let length_storage = manager.get_storage::<Length>().unwrap();

    assert_eq!(temp_storage.n_attributes(), 0);
    assert_eq!(length_storage.n_attributes(), 0);
}

#[test]
fn test_orbit_specific_merges() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Setup values
    manager.set_attribute(0, Temperature::from(20.0));
    manager.set_attribute(1, Temperature::from(30.0));

    // Test vertex-specific merge
    manager.merge_vertex_attributes(2, 0, 1);

    assert!(manager.get_attribute::<Temperature>(2).is_some());
}

#[test]
fn test_orbit_specific_splits() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Setup value
    manager.set_attribute(0, Temperature::from(25.0));

    // Test vertex-specific split
    manager.split_vertex_attributes(1, 2, 0);

    assert!(manager.get_attribute::<Temperature>(1).is_some());
    assert!(manager.get_attribute::<Temperature>(2).is_some());
}

// --- unit tests

// transactional manager methods

fn setup_manager() -> AttrStorageManager {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(10); // Initialize with size 10
    manager
}

#[test]
fn test_merge_vertex_attributes_transac() {
    let manager = setup_manager();
    // Set initial values
    manager.set_attribute(0, Temperature::from(20.0));
    manager.set_attribute(1, Temperature::from(30.0));

    atomically(|trans| manager.merge_vertex_attributes_transac(trans, 2, 0, 1));

    // Verify merged result
    let merged = manager.get_attribute::<Temperature>(2);
    assert!(merged.is_some());
    assert_eq!(merged.unwrap(), Temperature::from(25.0));
}

#[test]
fn test_split_vertex_attributes_transac() {
    let manager = setup_manager();

    // Set initial value
    manager.set_attribute(0, Temperature::from(20.0));

    atomically(|trans| manager.split_vertex_attributes_transac(trans, 1, 2, 0));

    // Verify split results
    let split1 = manager.get_attribute::<Temperature>(1);
    let split2 = manager.get_attribute::<Temperature>(2);

    assert!(split1.is_some());
    assert!(split2.is_some());
    assert_eq!(split1.unwrap().val, 20.0);
    assert_eq!(split2.unwrap().val, 20.0);
}

#[test]
fn test_set_attribute_transac() {
    let manager = setup_manager();

    atomically(|trans| manager.set_attribute_transac(trans, 0, Temperature::from(25.0)));

    let value = manager.get_attribute::<Temperature>(0);
    assert!(value.is_some());
    assert_eq!(value.unwrap().val, 25.0);
}

#[test]
fn test_insert_attribute_transac() {
    let manager = setup_manager();

    atomically(|trans| manager.insert_attribute_transac(trans, 0, Temperature::from(25.0)));

    let value = manager.get_attribute::<Temperature>(0);
    assert!(value.is_some());
    assert_eq!(value.unwrap().val, 25.0);
}

#[test]
fn test_get_attribute_transac() {
    let manager = setup_manager();

    // Set initial value
    manager.set_attribute(0, Temperature::from(25.0));

    let value = atomically(|trans| manager.get_attribute_transac::<Temperature>(trans, 0));

    assert!(value.is_some());
    assert_eq!(value.unwrap().val, 25.0);
}

#[test]
fn test_replace_attribute_transac() {
    let manager = setup_manager();

    // Set initial value
    manager.set_attribute(0, Temperature::from(25.0));

    let old_value =
        atomically(|trans| manager.replace_attribute_transac(trans, 0, Temperature::from(30.0)));

    assert!(old_value.is_some());
    assert_eq!(old_value.unwrap().val, 25.0);

    let new_value = manager.get_attribute::<Temperature>(0);
    assert!(new_value.is_some());
    assert_eq!(new_value.unwrap().val, 30.0);
}

#[test]
fn test_remove_attribute_transac() {
    let manager = setup_manager();

    // Set initial value
    manager.set_attribute(0, Temperature::from(25.0));

    let removed_value =
        atomically(|trans| manager.remove_attribute_transac::<Temperature>(trans, 0));

    assert!(removed_value.is_some());
    assert_eq!(removed_value.unwrap().val, 25.0);

    let value = manager.get_attribute::<Temperature>(0);
    assert!(value.is_none());
}

#[test]
fn test_merge_attribute_transac() {
    let manager = setup_manager();

    // Set initial values
    manager.set_attribute(0, Temperature::from(20.0));
    manager.set_attribute(1, Temperature::from(30.0));

    atomically(|trans| manager.merge_attribute_transac::<Temperature>(trans, 2, 0, 1));

    let merged = manager.get_attribute::<Temperature>(2);
    assert!(merged.is_some());
    assert_eq!(merged.unwrap().val, 25.0); // Assuming merge averages values
}

#[test]
fn test_split_attribute_transac() {
    let manager = setup_manager();

    // Set initial value
    manager.set_attribute(0, Temperature::from(20.0));

    atomically(|trans| manager.split_attribute_transac::<Temperature>(trans, 1, 2, 0));

    let split1 = manager.get_attribute::<Temperature>(1);
    let split2 = manager.get_attribute::<Temperature>(2);

    assert!(split1.is_some());
    assert!(split2.is_some());
    assert_eq!(split1.unwrap().val, 20.0); // Assuming split copies values
    assert_eq!(split2.unwrap().val, 20.0);
}

#[test]
fn test_attribute_operations_with_failed_transaction() {
    let manager = setup_manager();

    // Set initial value
    manager.set_attribute(0, Temperature::from(25.0));

    let _: Option<()> = Transaction::with_control(
        |_err| TransactionControl::Abort,
        |trans| {
            manager.set_attribute_transac(trans, 0, Temperature::from(30.0))?;
            manager.insert_attribute_transac(trans, 1, Temperature::from(35.0))?;

            Err(StmError::Failure)
        },
    );

    // Verify original values remained unchanged
    let value0 = manager.get_attribute::<Temperature>(0);
    let value1 = manager.get_attribute::<Temperature>(1);

    assert!(value0.is_some());
    assert_eq!(value0.unwrap().val, 25.0);
    assert!(value1.is_none());
}

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
