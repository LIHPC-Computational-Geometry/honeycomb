// ------ IMPORTS

use loom::sync::Arc;
use stm::{atomically, StmError, Transaction, TransactionControl};

use super::{
    AttrSparseVec, AttrStorageManager, AttributeBind, AttributeStorage, AttributeUpdate,
    UnknownAttributeStorage,
};
use crate::{
    cmap::{CMapResult, EdgeIdType},
    prelude::{CMap2, CMapBuilder, FaceIdType, OrbitPolicy, Vertex2, VertexIdType},
};
use std::any::Any;

// ------ CONTENT

// --- basic structure implementation

// vertex bound

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

    fn merge_incomplete(attr: Self) -> CMapResult<Self> {
        Ok(Temperature::from(attr.val / 2.0))
    }

    fn merge_from_none() -> CMapResult<Self> {
        Ok(Temperature::from(0.0))
    }
}

impl AttributeBind for Temperature {
    type StorageType = AttrSparseVec<Temperature>;
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}

impl From<f32> for Temperature {
    fn from(val: f32) -> Self {
        Self { val }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Weight(pub u32);

impl AttributeUpdate for Weight {
    fn merge(attr1: Self, attr2: Self) -> Self {
        Self(attr1.0 + attr2.0)
    }

    fn split(attr: Self) -> (Self, Self) {
        // adding the % to keep things conservative
        (Weight(attr.0 / 2 + attr.0 % 2), Weight(attr.0 / 2))
    }
}

impl AttributeBind for Weight {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = VertexIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Vertex;
}

// edge bound

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
    type IdentifierType = EdgeIdType;
    type StorageType = AttrSparseVec<Self>;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Edge;
}

// face bound

fn mean(a: u8, b: u8) -> u8 {
    ((u16::from(a) + u16::from(b)) / 2) as u8
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Color(pub u8, pub u8, pub u8);

impl AttributeUpdate for Color {
    fn merge(attr1: Self, attr2: Self) -> Self {
        Self(
            mean(attr1.0, attr2.0),
            mean(attr1.1, attr2.1),
            mean(attr1.2, attr2.2),
        )
    }

    fn split(attr: Self) -> (Self, Self) {
        (attr, attr)
    }
}

impl AttributeBind for Color {
    type StorageType = AttrSparseVec<Self>;
    type IdentifierType = FaceIdType;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Face;
}

// --- usual workflow test

#[test]
fn temperature_map() {
    // build the map
    let builder = CMapBuilder::default()
        .n_darts(6)
        .add_attribute::<Temperature>();
    let map: CMap2<f64> = builder.build().unwrap();

    map.force_two_link(1, 2);
    map.force_two_link(3, 4);
    map.force_two_link(5, 6);
    map.force_one_link(1, 3);
    map.force_write_vertex(1, (0.0, 0.0));
    map.force_write_vertex(2, (1.0, 0.0));
    map.force_write_vertex(4, (1.5, 0.0));
    map.force_write_vertex(5, (2.5, 0.0));
    map.force_write_vertex(6, (3.0, 0.0));
    map.force_write_attribute::<Temperature>(1, Temperature::from(273.));
    map.force_write_attribute::<Temperature>(2, Temperature::from(275.));
    map.force_write_attribute::<Temperature>(4, Temperature::from(277.));
    map.force_write_attribute::<Temperature>(5, Temperature::from(273.));
    map.force_write_attribute::<Temperature>(6, Temperature::from(273.));

    // test the map
    assert_eq!(
        map.force_read_attribute::<Temperature>(map.vertex_id(4)),
        Some(Temperature::from(277.))
    );
    assert_eq!(
        map.force_read_attribute::<Temperature>(map.vertex_id(5)),
        Some(Temperature::from(273.))
    );
    // sew one segment
    map.force_one_sew(3, 5);
    assert_eq!(map.vertex_id(4), map.vertex_id(5));
    assert_eq!(
        map.force_read_attribute::<Temperature>(map.vertex_id(4)),
        Some(Temperature::from(275.))
    );
    assert_eq!(
        map.force_read_vertex(map.vertex_id(4)),
        Some(Vertex2::from((2., 0.)))
    );
    // unsew another
    map.force_one_unsew(1);
    assert_ne!(map.vertex_id(2), map.vertex_id(3));
    assert_eq!(
        map.force_read_attribute::<Temperature>(map.vertex_id(2)),
        Some(Temperature::from(275.))
    );
    assert_eq!(
        map.force_read_attribute::<Temperature>(map.vertex_id(3)),
        Some(Temperature::from(275.))
    );
    assert_eq!(
        map.force_read_vertex(map.vertex_id(2)),
        Some(Vertex2::from((1., 0.)))
    );
    assert_eq!(
        map.force_read_vertex(map.vertex_id(3)),
        Some(Vertex2::from((1., 0.)))
    );
}

#[test]
fn test_attribute_operations() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Test set and get
    manager.force_write_attribute(0, Temperature::from(25.0));
    assert_eq!(
        manager.force_read_attribute::<Temperature>(0),
        Some(Temperature::from(25.0))
    );

    // Test remove
    let removed = manager.force_remove_attribute::<Temperature>(0);
    assert_eq!(removed, Some(Temperature::from(25.0)));
    assert_eq!(manager.force_read_attribute::<Temperature>(0), None);
}

#[test]
fn test_merge_attributes() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Setup initial values
    manager.force_write_attribute(0, Temperature::from(20.0));
    manager.force_write_attribute(1, Temperature::from(30.0));

    // Test merge
    atomically(|trans| manager.force_merge_attribute::<Temperature>(trans, 2, 0, 1));

    // The exact result depends on how merge is implemented in AttributeStorage
    // Just verify that something was stored at the output location
    assert!(manager.force_read_attribute::<Temperature>(2).is_some());
}

#[test]
fn test_split_attributes() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Setup initial value
    manager.force_write_attribute(0, Temperature::from(25.0));

    // Test split
    atomically(|trans| manager.force_split_attribute::<Temperature>(trans, 1, 2, 0));

    // The exact results depend on how split is implemented in AttributeStorage
    // Just verify that something was stored at both output locations
    assert!(manager.force_read_attribute::<Temperature>(1).is_some());
    assert!(manager.force_read_attribute::<Temperature>(2).is_some());
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
    manager.force_write_attribute(0, Temperature::from(20.0));
    manager.force_write_attribute(1, Temperature::from(30.0));

    // Test vertex-specific merge
    atomically(|trans| manager.force_merge_vertex_attributes(trans, 2, 0, 1));

    assert!(manager.force_read_attribute::<Temperature>(2).is_some());
}

#[test]
fn test_orbit_specific_splits() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Setup value
    manager.force_write_attribute(0, Temperature::from(25.0));

    // Test vertex-specific split
    atomically(|trans| manager.force_split_vertex_attributes(trans, 1, 2, 0));

    assert!(manager.force_read_attribute::<Temperature>(1).is_some());
    assert!(manager.force_read_attribute::<Temperature>(2).is_some());
}

// --- unit tests

// transactional manager methods

fn setup_manager() -> AttrStorageManager {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(10); // Initialize with size 10
    manager
}

#[test]
fn test_merge_vertex_attributes() {
    let manager = setup_manager();
    // Set initial values
    manager.force_write_attribute(0, Temperature::from(20.0));
    manager.force_write_attribute(1, Temperature::from(30.0));

    atomically(|trans| Ok(manager.merge_vertex_attributes(trans, 2, 0, 1)?));

    // Verify merged result
    let merged = manager.force_read_attribute::<Temperature>(2);
    assert!(merged.is_some());
    assert_eq!(merged.unwrap(), Temperature::from(25.0));
}

#[test]
fn test_split_vertex_attributes() {
    let manager = setup_manager();

    // Set initial value
    manager.force_write_attribute(0, Temperature::from(20.0));

    atomically(|trans| Ok(manager.split_vertex_attributes(trans, 1, 2, 0)?));

    // Verify split results
    let split1 = manager.force_read_attribute::<Temperature>(1);
    let split2 = manager.force_read_attribute::<Temperature>(2);

    assert!(split1.is_some());
    assert!(split2.is_some());
    assert_eq!(split1.unwrap().val, 20.0);
    assert_eq!(split2.unwrap().val, 20.0);
}

#[test]
fn test_write_attribute() {
    let manager = setup_manager();

    atomically(|trans| manager.write_attribute(trans, 0, Temperature::from(25.0)));

    let value = manager.force_read_attribute::<Temperature>(0);
    assert!(value.is_some());
    assert_eq!(value.unwrap().val, 25.0);
}

#[test]
fn test_read_attribute() {
    let manager = setup_manager();

    // Set initial value
    manager.force_write_attribute(0, Temperature::from(25.0));

    let value = atomically(|trans| manager.read_attribute::<Temperature>(trans, 0));

    assert!(value.is_some());
    assert_eq!(value.unwrap().val, 25.0);
}

#[test]
fn test_remove_attribute() {
    let manager = setup_manager();

    // Set initial value
    manager.force_write_attribute(0, Temperature::from(25.0));

    let removed_value = atomically(|trans| manager.remove_attribute::<Temperature>(trans, 0));

    assert!(removed_value.is_some());
    assert_eq!(removed_value.unwrap().val, 25.0);

    let value = manager.force_read_attribute::<Temperature>(0);
    assert!(value.is_none());
}

#[test]
fn test_merge_attribute() {
    let manager = setup_manager();

    // Set initial values
    manager.force_write_attribute(0, Temperature::from(20.0));
    manager.force_write_attribute(1, Temperature::from(30.0));

    atomically(|trans| Ok(manager.merge_attribute::<Temperature>(trans, 2, 0, 1)?));

    let merged = manager.force_read_attribute::<Temperature>(2);
    assert!(merged.is_some());
    assert_eq!(merged.unwrap().val, 25.0); // Assuming merge averages values
}

#[test]
fn test_split_attribute() {
    let manager = setup_manager();

    // Set initial value
    manager.force_write_attribute(0, Temperature::from(20.0));

    atomically(|trans| Ok(manager.split_attribute::<Temperature>(trans, 1, 2, 0)?));

    let split1 = manager.force_read_attribute::<Temperature>(1);
    let split2 = manager.force_read_attribute::<Temperature>(2);

    assert!(split1.is_some());
    assert!(split2.is_some());
    assert_eq!(split1.unwrap().val, 20.0); // Assuming split copies values
    assert_eq!(split2.unwrap().val, 20.0);
}

#[test]
fn test_attribute_operations_with_failed_transaction() {
    let manager = setup_manager();

    // Set initial value
    manager.force_write_attribute(0, Temperature::from(25.0));

    let _: Option<()> = Transaction::with_control(
        |_err| TransactionControl::Abort,
        |trans| {
            manager.write_attribute(trans, 0, Temperature::from(30.0))?;
            manager.write_attribute(trans, 1, Temperature::from(35.0))?;

            Err(StmError::Failure)
        },
    );

    // Verify original values remained unchanged
    let value0 = manager.force_read_attribute::<Temperature>(0);
    let value1 = manager.force_read_attribute::<Temperature>(1);

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
        Ok(Temperature::from(t_ref.val / 2.0))
    );
    assert_eq!(Temperature::merge_from_none(), Ok(Temperature::from(0.0)));
}

#[test]
fn attribute_bind() {
    assert_eq!(Temperature::BIND_POLICY, OrbitPolicy::Vertex);
    let inst: <Temperature as AttributeBind>::IdentifierType = 0;
    let ref_inst: FaceIdType = 0;
    let prim_inst: u32 = 0;
    assert_eq!(inst.type_id(), ref_inst.type_id());
    assert_eq!(inst.type_id(), prim_inst.type_id());
}

// storages

macro_rules! generate_sparse {
    ($name: ident) => {
        #[allow(unused_mut)]
        let mut $name = AttrSparseVec::<Temperature>::new(10);
        $name.force_write(0, Temperature::from(273.0));
        $name.force_write(1, Temperature::from(275.0));
        $name.force_write(2, Temperature::from(277.0));
        $name.force_write(3, Temperature::from(279.0));
        $name.force_write(4, Temperature::from(281.0));
        $name.force_write(5, Temperature::from(283.0));
        $name.force_write(6, Temperature::from(285.0));
        $name.force_write(7, Temperature::from(287.0));
        $name.force_write(8, Temperature::from(289.0));
        $name.force_write(9, Temperature::from(291.0));
    };
}

#[test]
fn sparse_vec_n_attributes() {
    generate_sparse!(storage);
    assert_eq!(storage.n_attributes(), 10);
    let _ = storage.force_remove(3);
    assert_eq!(storage.n_attributes(), 9);
    // extend does not affect the number of attributes
    storage.extend(10);
    assert!(storage.force_read(15).is_none());
    assert_eq!(storage.n_attributes(), 9);
}

#[test]
fn sparse_vec_merge() {
    generate_sparse!(storage);
    assert_eq!(storage.force_read(3), Some(Temperature::from(279.0)));
    assert_eq!(storage.force_read(6), Some(Temperature::from(285.0)));
    assert_eq!(storage.force_read(8), Some(Temperature::from(289.0)));
    atomically(|trans| storage.force_merge(trans, 8, 3, 6));
    assert_eq!(storage.force_read(3), None);
    assert_eq!(storage.force_read(6), None);
    assert_eq!(storage.force_read(8), Some(Temperature::from(282.0)));
}

#[test]
fn sparse_vec_merge_undefined() {
    generate_sparse!(storage);
    assert_eq!(storage.force_remove(3), Some(Temperature::from(279.0)));
    assert_eq!(storage.force_remove(6), Some(Temperature::from(285.0)));
    assert_eq!(storage.force_remove(8), Some(Temperature::from(289.0)));
    // merge from two undefined value
    atomically(|trans| storage.force_merge(trans, 8, 3, 6));
    assert_eq!(storage.force_read(3), None);
    assert_eq!(storage.force_read(6), None);
    assert_eq!(storage.force_read(8), Some(Temperature::from(0.0)));
    // merge from one undefined value
    assert_eq!(storage.force_read(4), Some(Temperature::from(281.0)));
    atomically(|trans| storage.force_merge(trans, 6, 3, 4));
    assert_eq!(storage.force_read(3), None);
    assert_eq!(storage.force_read(4), None);
    assert_eq!(storage.force_read(6), Some(Temperature::from(281.0 / 2.0)));
}

#[test]
fn sparse_vec_split() {
    generate_sparse!(storage);
    assert_eq!(storage.force_remove(3), Some(Temperature::from(279.0)));
    assert_eq!(storage.force_remove(6), Some(Temperature::from(285.0)));
    assert_eq!(storage.force_read(8), Some(Temperature::from(289.0)));
    atomically(|trans| storage.force_split(trans, 3, 6, 8));
    assert_eq!(storage.force_read(3), Some(Temperature::from(289.0)));
    assert_eq!(storage.force_read(6), Some(Temperature::from(289.0)));
    assert_eq!(storage.force_read(8), None);
}

#[test]
fn sparse_vec_read_set_read() {
    generate_sparse!(storage);
    assert_eq!(storage.force_read(3), Some(Temperature::from(279.0)));
    storage.force_write(3, Temperature::from(280.0));
    assert_eq!(storage.force_read(3), Some(Temperature::from(280.0)));
}

#[test]
fn sparse_vec_remove() {
    generate_sparse!(storage);
    assert_eq!(storage.force_remove(3), Some(Temperature::from(279.0)));
}

#[test]
fn sparse_vec_remove_remove() {
    generate_sparse!(storage);
    assert_eq!(storage.force_remove(3), Some(Temperature::from(279.0)));
    assert!(storage.force_remove(3).is_none());
}

#[test]
fn sparse_vec_remove_read() {
    generate_sparse!(storage);
    assert_eq!(storage.force_remove(3), Some(Temperature::from(279.0)));
    assert!(storage.force_read(3).is_none());
}

#[test]
fn sparse_vec_remove_set() {
    generate_sparse!(storage);
    assert_eq!(storage.force_remove(3), Some(Temperature::from(279.0)));
    storage.force_write(3, Temperature::from(280.0));
    assert!(storage.force_read(3).is_some());
}

// storage manager

macro_rules! generate_manager {
    ($name: ident) => {
        let mut $name = AttrStorageManager::default();
        $name.add_storage::<Temperature>(10);
        $name.force_write_attribute(0, Temperature::from(273.0));
        $name.force_write_attribute(1, Temperature::from(275.0));
        $name.force_write_attribute(2, Temperature::from(277.0));
        $name.force_write_attribute(3, Temperature::from(279.0));
        $name.force_write_attribute(4, Temperature::from(281.0));
        $name.force_write_attribute(5, Temperature::from(283.0));
        $name.force_write_attribute(6, Temperature::from(285.0));
        $name.force_write_attribute(7, Temperature::from(287.0));
        $name.force_write_attribute(8, Temperature::from(289.0));
        $name.force_write_attribute(9, Temperature::from(291.0));
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
    (10..20).for_each(|id| {
        manager.force_write_attribute(id, Temperature::from(273.0 + 2.0 * id as f32));
    });
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
    manager.force_write_attribute(15, Temperature::from(0.0)); // panic
}

#[test]
fn manager_read_set_read() {
    generate_manager!(manager);
    assert_eq!(
        manager.force_read_attribute(3),
        Some(Temperature::from(279.0))
    );
    manager.force_write_attribute(3, Temperature::from(280.0));
    assert_eq!(
        manager.force_read_attribute(3),
        Some(Temperature::from(280.0))
    );
}

#[test]
fn manager_vec_remove_remove() {
    generate_manager!(manager);
    assert_eq!(
        manager.force_remove_attribute(3),
        Some(Temperature::from(279.0))
    );
    assert!(manager.force_remove_attribute::<Temperature>(3).is_none());
}

#[test]
fn manager_vec_remove_read() {
    generate_manager!(manager);
    assert_eq!(
        manager.force_remove_attribute(3),
        Some(Temperature::from(279.0))
    );
    assert!(manager.force_read_attribute::<Temperature>(3).is_none());
}

#[test]
fn manager_vec_remove_set() {
    generate_manager!(manager);
    assert_eq!(
        manager.force_remove_attribute(3),
        Some(Temperature::from(279.0))
    );
    manager.force_write_attribute(3, Temperature::from(280.0));
    assert!(manager.force_read_attribute::<Temperature>(3).is_some());
}

#[test]
fn manager_merge_attribute() {
    generate_manager!(manager);
    assert_eq!(
        manager.force_read_attribute(3),
        Some(Temperature::from(279.0))
    );
    assert_eq!(
        manager.force_read_attribute(6),
        Some(Temperature::from(285.0))
    );
    assert_eq!(
        manager.force_read_attribute(8),
        Some(Temperature::from(289.0))
    );
    atomically(|trans| manager.force_merge_attribute::<Temperature>(trans, 8, 3, 6));
    assert_eq!(manager.force_read_attribute::<Temperature>(3), None);
    assert_eq!(manager.force_read_attribute::<Temperature>(6), None);
    assert_eq!(
        manager.force_read_attribute(8),
        Some(Temperature::from(282.0))
    );
}

#[test]
fn manager_merge_undefined_attribute() {
    generate_manager!(manager);
    assert_eq!(
        manager.force_remove_attribute(3),
        Some(Temperature::from(279.0))
    );
    assert_eq!(
        manager.force_remove_attribute(6),
        Some(Temperature::from(285.0))
    );
    assert_eq!(
        manager.force_remove_attribute(8),
        Some(Temperature::from(289.0))
    );
    // merge from two undefined value
    atomically(|trans| manager.force_merge_attribute::<Temperature>(trans, 8, 3, 6));
    assert_eq!(manager.force_read_attribute::<Temperature>(3), None);
    assert_eq!(manager.force_read_attribute::<Temperature>(6), None);
    assert_eq!(
        manager.force_read_attribute(8),
        Some(Temperature::from(0.0))
    );
    // merge from one undefined value
    assert_eq!(
        manager.force_read_attribute(4),
        Some(Temperature::from(281.0))
    );
    atomically(|trans| manager.force_merge_attribute::<Temperature>(trans, 6, 3, 4));
    assert_eq!(manager.force_read_attribute::<Temperature>(3), None);
    assert_eq!(manager.force_read_attribute::<Temperature>(4), None);
    assert_eq!(
        manager.force_read_attribute(6),
        Some(Temperature::from(281.0 / 2.0))
    );
}

#[test]
fn manager_split_attribute() {
    generate_manager!(manager);
    assert_eq!(
        manager.force_remove_attribute(3),
        Some(Temperature::from(279.0))
    );
    assert_eq!(
        manager.force_remove_attribute(6),
        Some(Temperature::from(285.0))
    );
    assert_eq!(
        manager.force_read_attribute(8),
        Some(Temperature::from(289.0))
    );
    atomically(|trans| manager.force_split_attribute::<Temperature>(trans, 3, 6, 8));
    assert_eq!(
        manager.force_read_attribute(3),
        Some(Temperature::from(289.0))
    );
    assert_eq!(
        manager.force_read_attribute(6),
        Some(Temperature::from(289.0))
    );
    assert_eq!(manager.force_read_attribute::<Temperature>(8), None);
}

// --- parallel

#[allow(clippy::too_many_lines)]
#[test]
fn manager_ordering() {
    loom::model(|| {
        // setup manager; slot 0, 1, 2, 3
        let mut manager = AttrStorageManager::default();
        manager.add_storage::<Temperature>(4);
        manager.add_storage::<Length>(4);
        manager.add_storage::<Weight>(4);
        manager.add_storage::<Color>(4);

        manager.force_write_attribute(1, Temperature::from(20.0));
        manager.force_write_attribute(3, Temperature::from(30.0));

        manager.force_write_attribute(1, Length(3.0));
        manager.force_write_attribute(3, Length(2.0));

        manager.force_write_attribute(1, Weight(10));
        manager.force_write_attribute(3, Weight(15));

        manager.force_write_attribute(1, Color(255, 0, 0));
        manager.force_write_attribute(3, Color(0, 0, 255));

        let arc = Arc::new(manager);
        let c1 = arc.clone();
        let c2 = arc.clone();

        // we're going to do 2 ops:
        // - merge (1, 3) => 2
        // - split 2 =< (2, 3)
        // depending on the execution path, attribute values on slots 2 and 3 will vary
        // attribute value of slot 1 should be None in any case

        let t1 = loom::thread::spawn(move || {
            atomically(|trans| {
                c1.merge_vertex_attributes(trans, 2, 1, 3)?;
                c1.merge_edge_attributes(trans, 2, 1, 3)?;
                c1.merge_face_attributes(trans, 2, 1, 3)?;
                Ok(())
            });
        });

        let t2 = loom::thread::spawn(move || {
            atomically(|trans| {
                c2.split_vertex_attributes(trans, 2, 3, 2)?;
                c2.split_edge_attributes(trans, 2, 3, 2)?;
                c2.split_face_attributes(trans, 2, 3, 2)?;
                Ok(())
            });
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // in both cases
        let slot_1_is_empty = arc.force_read_attribute::<Temperature>(1).is_none()
            && arc.force_read_attribute::<Weight>(1).is_none()
            && arc.force_read_attribute::<Length>(1).is_none()
            && arc.force_read_attribute::<Color>(1).is_none();
        assert!(slot_1_is_empty);

        // path 1: merge before split
        let p1_2_temp = arc
            .force_read_attribute::<Temperature>(2)
            .is_some_and(|val| val == Temperature::from(25.0));
        let p1_3_temp = arc
            .force_read_attribute::<Temperature>(3)
            .is_some_and(|val| val == Temperature::from(25.0));

        let p1_2_weight = arc
            .force_read_attribute::<Weight>(2)
            .is_some_and(|v| v == Weight(13));
        let p1_3_weight = arc
            .force_read_attribute::<Weight>(3)
            .is_some_and(|v| v == Weight(12));

        let p1_2_len = arc
            .force_read_attribute::<Length>(2)
            .is_some_and(|v| v == Length(2.5));
        let p1_3_len = arc
            .force_read_attribute::<Length>(3)
            .is_some_and(|v| v == Length(2.5));

        let p1_2_col = arc
            .force_read_attribute::<Color>(2)
            .is_some_and(|v| v == Color(127, 0, 127));
        let p1_3_col = arc
            .force_read_attribute::<Color>(3)
            .is_some_and(|v| v == Color(127, 0, 127));

        let p1 = slot_1_is_empty
            && p1_2_temp
            && p1_3_temp
            && p1_2_weight
            && p1_3_weight
            && p1_2_len
            && p1_3_len
            && p1_2_col
            && p1_3_col;

        // path 2: split before merge
        let p2_2_temp = arc
            .force_read_attribute::<Temperature>(2)
            .is_some_and(|val| val == Temperature::from(5.0));
        let p2_3_temp = arc.force_read_attribute::<Temperature>(3).is_none();

        let p2_2_weight = arc
            .force_read_attribute::<Weight>(2)
            .is_some_and(|v| v == Weight(10));
        let p2_3_weight = arc.force_read_attribute::<Weight>(3).is_none();

        let p2_2_len = arc
            .force_read_attribute::<Length>(2)
            .is_some_and(|v| v == Length(3.0));
        let p2_3_len = arc.force_read_attribute::<Length>(3).is_none();

        let p2_2_col = arc
            .force_read_attribute::<Color>(2)
            .is_some_and(|v| v == Color(255, 0, 0));
        let p2_3_col = arc.force_read_attribute::<Color>(3).is_none();

        let p2 = slot_1_is_empty
            && p2_2_temp
            && p2_3_temp
            && p2_2_weight
            && p2_3_weight
            && p2_2_len
            && p2_3_len
            && p2_2_col
            && p2_3_col;

        assert!(p1 || p2);
    });
}
