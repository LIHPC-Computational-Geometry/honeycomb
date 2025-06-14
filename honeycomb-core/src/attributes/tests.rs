use std::any::Any;

use loom::sync::Arc;

use crate::{
    attributes::{
        AttrSparseVec, AttrStorageManager, AttributeBind, AttributeError, AttributeStorage,
        AttributeUpdate, UnknownAttributeStorage,
    },
    cmap::{CMap2, CMapBuilder, EdgeIdType, FaceIdType, OrbitPolicy, VertexIdType},
    geometry::Vertex2,
    stm::{StmError, Transaction, TransactionControl, atomically},
};

// --- basic structure implementation

// vertex bound

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Temperature {
    pub val: f32,
}

impl AttributeUpdate for Temperature {
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        Ok(Temperature {
            val: f32::midpoint(attr1.val, attr2.val),
        })
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }

    fn merge_incomplete(attr: Self) -> Result<Self, AttributeError> {
        Ok(Temperature::from(attr.val / 2.0))
    }

    fn merge_from_none() -> Result<Self, AttributeError> {
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
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        Ok(Self(attr1.0 + attr2.0))
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        // adding the % to keep things conservative
        Ok((Weight(attr.0 / 2 + attr.0 % 2), Weight(attr.0 / 2)))
    }

    fn merge_incomplete(attr: Self) -> Result<Self, AttributeError> {
        Ok(attr)
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
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        Ok(Length(f32::midpoint(attr1.0, attr2.0)))
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }

    fn merge_incomplete(attr: Self) -> Result<Self, AttributeError> {
        Ok(attr)
    }
}

impl AttributeBind for Length {
    type IdentifierType = EdgeIdType;
    type StorageType = AttrSparseVec<Self>;
    const BIND_POLICY: OrbitPolicy = OrbitPolicy::Edge;
}

// face bound

fn mean(a: u8, b: u8) -> u8 {
    u16::midpoint(u16::from(a), u16::from(b)) as u8
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
struct Color(pub u8, pub u8, pub u8);

impl AttributeUpdate for Color {
    fn merge(attr1: Self, attr2: Self) -> Result<Self, AttributeError> {
        Ok(Self(
            mean(attr1.0, attr2.0),
            mean(attr1.1, attr2.1),
            mean(attr1.2, attr2.2),
        ))
    }

    fn split(attr: Self) -> Result<(Self, Self), AttributeError> {
        Ok((attr, attr))
    }

    fn merge_incomplete(attr: Self) -> Result<Self, AttributeError> {
        Ok(attr)
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
    let builder = CMapBuilder::<2, _>::from_n_darts(6).add_attribute::<Temperature>();
    let map: CMap2<f64> = builder.build().unwrap();

    map.force_link::<2>(1, 2).unwrap();
    map.force_link::<2>(3, 4).unwrap();
    map.force_link::<2>(5, 6).unwrap();
    map.force_link::<1>(1, 3).unwrap();
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
    map.force_sew::<1>(3, 5).unwrap();
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
    map.force_unsew::<1>(1).unwrap();
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

    atomically(|t| {
        // Test set and get
        manager.write_attribute(t, 0, Temperature::from(25.0))?;
        assert_eq!(
            manager.read_attribute::<Temperature>(t, 0)?,
            Some(Temperature::from(25.0))
        );

        // Test remove
        let removed = manager.remove_attribute::<Temperature>(t, 0)?;
        assert_eq!(removed, Some(Temperature::from(25.0)));
        assert_eq!(manager.read_attribute::<Temperature>(t, 0)?, None);
        Ok(())
    });
}

#[test]
fn test_merge_attributes() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Setup initial values
    atomically(|t| {
        manager.write_attribute(t, 0, Temperature::from(20.0))?;
        manager.write_attribute(t, 1, Temperature::from(30.0))?;
        Ok(())
    });

    // Test merge
    atomically(|trans| {
        manager
            .merge_attribute::<Temperature>(trans, 2, 0, 1)
            .map_err(|_| StmError::Failure)
    });

    // The exact result depends on how merge is implemented in AttributeStorage
    // Just verify that something was stored at the output location
    atomically(|t| {
        assert!(manager.read_attribute::<Temperature>(t, 2)?.is_some());
        Ok(())
    });
}

#[test]
fn test_split_attributes() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Setup initial value
    atomically(|t| manager.write_attribute(t, 0, Temperature::from(25.0)));

    // Test split
    atomically(|trans| {
        manager
            .split_attribute::<Temperature>(trans, 1, 2, 0)
            .map_err(|_| StmError::Failure)
    });

    // The exact results depend on how split is implemented in AttributeStorage
    // Just verify that something was stored at both output locations
    atomically(|t| {
        assert!(manager.read_attribute::<Temperature>(t, 1)?.is_some());
        assert!(manager.read_attribute::<Temperature>(t, 2)?.is_some());
        Ok(())
    });
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
    atomically(|t| {
        manager.write_attribute(t, 0, Temperature::from(20.0))?;
        manager.write_attribute(t, 1, Temperature::from(30.0))?;

        Ok(())
    });

    // Test vertex-specific merge
    atomically(|trans| {
        manager
            .merge_attributes(trans, OrbitPolicy::Vertex, 2, 0, 1)
            .map_err(|_| StmError::Failure)
    });

    atomically(|t| {
        assert!(manager.read_attribute::<Temperature>(t, 2)?.is_some());
        Ok(())
    });
}

#[test]
fn test_orbit_specific_splits() {
    let mut manager = AttrStorageManager::default();
    manager.add_storage::<Temperature>(5);

    // Setup value
    atomically(|t| manager.write_attribute(t, 0, Temperature::from(25.0)));

    // Test vertex-specific split
    atomically(|trans| {
        manager
            .split_attributes(trans, OrbitPolicy::Vertex, 1, 2, 0)
            .map_err(|_| StmError::Failure)
    });

    atomically(|t| {
        assert!(manager.read_attribute::<Temperature>(t, 1)?.is_some());
        assert!(manager.read_attribute::<Temperature>(t, 2)?.is_some());

        Ok(())
    });
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
    atomically(|t| {
        manager.write_attribute(t, 0, Temperature::from(20.0))?;
        manager.write_attribute(t, 1, Temperature::from(30.0))?;

        Ok(())
    });

    atomically(|trans| {
        manager
            .merge_attributes(trans, OrbitPolicy::Vertex, 2, 0, 1)
            .map_err(|_| StmError::Failure)
    });

    // Verify merged result
    let merged = atomically(|t| manager.read_attribute::<Temperature>(t, 2));
    assert!(merged.is_some());
    assert_eq!(merged.unwrap(), Temperature::from(25.0));
}

#[test]
fn test_split_vertex_attributes() {
    let manager = setup_manager();

    // Set initial value
    atomically(|t| manager.write_attribute(t, 0, Temperature::from(20.0)));

    atomically(|trans| {
        manager
            .split_attributes(trans, OrbitPolicy::Vertex, 1, 2, 0)
            .map_err(|_| StmError::Failure)
    });

    // Verify split results
    atomically(|t| {
        let split1 = manager.read_attribute::<Temperature>(t, 1)?;
        let split2 = manager.read_attribute::<Temperature>(t, 2)?;

        assert!(split1.is_some());
        assert!(split2.is_some());
        assert_eq!(split1.unwrap().val, 20.0);
        assert_eq!(split2.unwrap().val, 20.0);

        Ok(())
    });
}

#[test]
fn test_write_attribute() {
    let manager = setup_manager();

    atomically(|trans| manager.write_attribute(trans, 0, Temperature::from(25.0)));

    let value = atomically(|t| manager.read_attribute::<Temperature>(t, 0));
    assert!(value.is_some());
    assert_eq!(value.unwrap().val, 25.0);
}

#[test]
fn test_read_attribute() {
    let manager = setup_manager();

    // Set initial value
    atomically(|t| manager.write_attribute(t, 0, Temperature::from(25.0)));

    let value = atomically(|trans| manager.read_attribute::<Temperature>(trans, 0));

    assert!(value.is_some());
    assert_eq!(value.unwrap().val, 25.0);
}

#[test]
fn test_remove_attribute() {
    let manager = setup_manager();

    // Set initial value
    atomically(|t| manager.write_attribute(t, 0, Temperature::from(25.0)));

    let removed_value = atomically(|trans| manager.remove_attribute::<Temperature>(trans, 0));

    assert!(removed_value.is_some());
    assert_eq!(removed_value.unwrap().val, 25.0);

    let value = atomically(|t| manager.read_attribute::<Temperature>(t, 0));
    assert!(value.is_none());
}

#[test]
fn test_merge_attribute() {
    let manager = setup_manager();

    // Set initial values
    atomically(|t| {
        manager.write_attribute(t, 0, Temperature::from(20.0))?;
        manager.write_attribute(t, 1, Temperature::from(30.0))?;

        Ok(())
    });

    atomically(|trans| {
        manager
            .merge_attribute::<Temperature>(trans, 2, 0, 1)
            .map_err(|_| StmError::Failure)
    });

    let merged = atomically(|t| manager.read_attribute::<Temperature>(t, 2));
    assert!(merged.is_some());
    assert_eq!(merged.unwrap().val, 25.0); // Assuming merge averages values
}

#[test]
fn test_split_attribute() {
    let manager = setup_manager();

    // Set initial value
    atomically(|t| manager.write_attribute(t, 0, Temperature::from(20.0)));

    atomically(|trans| {
        manager
            .split_attribute::<Temperature>(trans, 1, 2, 0)
            .map_err(|_| StmError::Failure)
    });

    atomically(|t| {
        let split1 = manager.read_attribute::<Temperature>(t, 1)?;
        let split2 = manager.read_attribute::<Temperature>(t, 2)?;

        assert!(split1.is_some());
        assert!(split2.is_some());
        assert_eq!(split1.unwrap().val, 20.0); // Assuming split copies values
        assert_eq!(split2.unwrap().val, 20.0);

        Ok(())
    });
}

#[test]
fn test_attribute_operations_with_failed_transaction() {
    let manager = setup_manager();

    // Set initial value
    atomically(|t| manager.write_attribute(t, 0, Temperature::from(25.0)));

    let _: Option<()> = Transaction::with_control(
        |_err| TransactionControl::Abort,
        |trans| {
            manager.write_attribute(trans, 0, Temperature::from(30.0))?;
            manager.write_attribute(trans, 1, Temperature::from(35.0))?;

            Err(StmError::Failure)
        },
    );

    // Verify original values remained unchanged
    atomically(|t| {
        let value0 = manager.read_attribute::<Temperature>(t, 0)?;
        let value1 = manager.read_attribute::<Temperature>(t, 1)?;

        assert!(value0.is_some());
        assert_eq!(value0.unwrap().val, 25.0);
        assert!(value1.is_none());

        Ok(())
    });
}

// traits

#[test]
fn attribute_update() {
    let t1 = Temperature { val: 273.0 };
    let t2 = Temperature { val: 298.0 };

    let t_new = AttributeUpdate::merge(t1, t2).unwrap(); // use AttributeUpdate::_
    let t_ref = Temperature { val: 285.5 };

    assert_eq!(Temperature::split(t_new), Ok((t_ref, t_ref))); // or Temperature::_
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
        atomically(|t| {
            $name.write(t, 0, Temperature::from(273.0))?;
            $name.write(t, 1, Temperature::from(275.0))?;
            $name.write(t, 2, Temperature::from(277.0))?;
            $name.write(t, 3, Temperature::from(279.0))?;
            $name.write(t, 4, Temperature::from(281.0))?;
            $name.write(t, 5, Temperature::from(283.0))?;
            $name.write(t, 6, Temperature::from(285.0))?;
            $name.write(t, 7, Temperature::from(287.0))?;
            $name.write(t, 8, Temperature::from(289.0))?;
            $name.write(t, 9, Temperature::from(291.0))?;
            Ok(())
        });
    };
}

#[test]
fn sparse_vec_n_attributes() {
    generate_sparse!(storage);
    assert_eq!(storage.n_attributes(), 10);
    let _ = atomically(|t| storage.remove(t, 3));
    assert_eq!(storage.n_attributes(), 9);
    // extend does not affect the number of attributes
    storage.extend(10);
    assert!(atomically(|t| storage.read(t, 15)).is_none());
    assert_eq!(storage.n_attributes(), 9);
}

#[test]
fn sparse_vec_merge() {
    generate_sparse!(storage);
    assert_eq!(
        atomically(|t| storage.read(t, 3)),
        Some(Temperature::from(279.0))
    );
    assert_eq!(
        atomically(|t| storage.read(t, 6)),
        Some(Temperature::from(285.0))
    );
    assert_eq!(
        atomically(|t| storage.read(t, 8)),
        Some(Temperature::from(289.0))
    );
    atomically(|t| storage.merge(t, 8, 3, 6).map_err(|_| StmError::Failure));
    assert_eq!(atomically(|t| storage.read(t, 3)), None);
    assert_eq!(atomically(|t| storage.read(t, 6)), None);
    assert_eq!(
        atomically(|t| storage.read(t, 8)),
        Some(Temperature::from(282.0))
    );
}

#[test]
fn sparse_vec_merge_undefined() {
    generate_sparse!(storage);
    assert_eq!(
        atomically(|t| storage.remove(t, 3)),
        Some(Temperature::from(279.0))
    );
    assert_eq!(
        atomically(|t| storage.remove(t, 6)),
        Some(Temperature::from(285.0))
    );
    assert_eq!(
        atomically(|t| storage.remove(t, 8)),
        Some(Temperature::from(289.0))
    );
    // merge from two undefined value
    atomically(|t| storage.merge(t, 8, 3, 6).map_err(|_| StmError::Failure));
    assert_eq!(atomically(|t| storage.read(t, 3)), None);
    assert_eq!(atomically(|t| storage.read(t, 6)), None);
    assert_eq!(
        atomically(|t| storage.read(t, 8)),
        Some(Temperature::from(0.0))
    );
    // merge from one undefined value
    assert_eq!(
        atomically(|t| storage.read(t, 4)),
        Some(Temperature::from(281.0))
    );
    atomically(|t| storage.merge(t, 6, 3, 4).map_err(|_| StmError::Failure));
    assert_eq!(atomically(|t| storage.read(t, 3)), None);
    assert_eq!(atomically(|t| storage.read(t, 4)), None);
    assert_eq!(
        atomically(|t| storage.read(t, 6)),
        Some(Temperature::from(281.0 / 2.0))
    );
}

#[test]
fn sparse_vec_split() {
    generate_sparse!(storage);
    assert_eq!(
        atomically(|t| storage.remove(t, 3)),
        Some(Temperature::from(279.0))
    );
    assert_eq!(
        atomically(|t| storage.remove(t, 6)),
        Some(Temperature::from(285.0))
    );
    assert_eq!(
        atomically(|t| storage.read(t, 8)),
        Some(Temperature::from(289.0))
    );
    atomically(|t| storage.split(t, 3, 6, 8).map_err(|_| StmError::Failure));
    assert_eq!(
        atomically(|t| storage.read(t, 3)),
        Some(Temperature::from(289.0))
    );
    assert_eq!(
        atomically(|t| storage.read(t, 6)),
        Some(Temperature::from(289.0))
    );
    assert_eq!(atomically(|t| storage.read(t, 8)), None);
}

#[test]
fn sparse_vec_read_set_read() {
    generate_sparse!(storage);
    assert_eq!(
        atomically(|t| storage.read(t, 3)),
        Some(Temperature::from(279.0))
    );
    atomically(|t| storage.write(t, 3, Temperature::from(280.0)));
    assert_eq!(
        atomically(|t| storage.read(t, 3)),
        Some(Temperature::from(280.0))
    );
}

#[test]
fn sparse_vec_remove() {
    generate_sparse!(storage);
    assert_eq!(
        atomically(|t| storage.remove(t, 3)),
        Some(Temperature::from(279.0))
    );
}

#[test]
fn sparse_vec_remove_remove() {
    generate_sparse!(storage);
    assert_eq!(
        atomically(|t| storage.remove(t, 3)),
        Some(Temperature::from(279.0))
    );
    assert!(atomically(|t| storage.remove(t, 3)).is_none());
}

#[test]
fn sparse_vec_remove_read() {
    generate_sparse!(storage);
    assert_eq!(
        atomically(|t| storage.remove(t, 3)),
        Some(Temperature::from(279.0))
    );
    assert!(atomically(|t| storage.read(t, 3)).is_none());
}

#[test]
fn sparse_vec_remove_set() {
    generate_sparse!(storage);
    assert_eq!(
        atomically(|t| storage.remove(t, 3)),
        Some(Temperature::from(279.0))
    );
    atomically(|t| storage.write(t, 3, Temperature::from(280.0)));
    assert!(atomically(|t| storage.read(t, 3)).is_some());
}

// storage manager

macro_rules! generate_manager {
    ($name: ident) => {
        let mut $name = AttrStorageManager::default();
        $name.add_storage::<Temperature>(10);
        atomically(|t| {
            $name.write_attribute(t, 0, Temperature::from(273.0))?;
            $name.write_attribute(t, 1, Temperature::from(275.0))?;
            $name.write_attribute(t, 2, Temperature::from(277.0))?;
            $name.write_attribute(t, 3, Temperature::from(279.0))?;
            $name.write_attribute(t, 4, Temperature::from(281.0))?;
            $name.write_attribute(t, 5, Temperature::from(283.0))?;
            $name.write_attribute(t, 6, Temperature::from(285.0))?;
            $name.write_attribute(t, 7, Temperature::from(287.0))?;
            $name.write_attribute(t, 8, Temperature::from(289.0))?;
            $name.write_attribute(t, 9, Temperature::from(291.0))?;
            Ok(())
        });
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
    atomically(|t| {
        for id in 10..20 {
            manager.write_attribute(t, id, Temperature::from(273.0 + 2.0 * id as f32))?;
        }
        Ok(())
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
    atomically(|t| manager.write_attribute(t, 15, Temperature::from(0.0))); // panic
}

#[test]
fn manager_read_set_read() {
    generate_manager!(manager);
    atomically(|t| {
        assert_eq!(
            manager.read_attribute(t, 3)?,
            Some(Temperature::from(279.0))
        );
        manager.write_attribute(t, 3, Temperature::from(280.0))?;
        assert_eq!(
            manager.read_attribute(t, 3)?,
            Some(Temperature::from(280.0))
        );
        Ok(())
    });
}

#[test]
fn manager_vec_remove_remove() {
    generate_manager!(manager);
    atomically(|t| {
        assert_eq!(
            manager.remove_attribute(t, 3)?,
            Some(Temperature::from(279.0))
        );
        assert!(manager.remove_attribute::<Temperature>(t, 3)?.is_none());
        Ok(())
    });
}

#[test]
fn manager_vec_remove_read() {
    generate_manager!(manager);
    atomically(|t| {
        assert_eq!(
            manager.remove_attribute(t, 3)?,
            Some(Temperature::from(279.0))
        );
        assert!(manager.read_attribute::<Temperature>(t, 3)?.is_none());
        Ok(())
    });
}

#[test]
fn manager_vec_remove_set() {
    generate_manager!(manager);
    atomically(|t| {
        assert_eq!(
            manager.remove_attribute(t, 3)?,
            Some(Temperature::from(279.0))
        );
        manager.write_attribute(t, 3, Temperature::from(280.0))?;
        assert!(manager.read_attribute::<Temperature>(t, 3)?.is_some());
        Ok(())
    });
}

#[test]
fn manager_merge_attribute() {
    generate_manager!(manager);
    atomically(|t| {
        assert_eq!(
            manager.read_attribute(t, 3)?,
            Some(Temperature::from(279.0))
        );
        assert_eq!(
            manager.read_attribute(t, 6)?,
            Some(Temperature::from(285.0))
        );
        assert_eq!(
            manager.read_attribute(t, 8)?,
            Some(Temperature::from(289.0))
        );
        manager
            .merge_attribute::<Temperature>(t, 8, 3, 6)
            .map_err(|_| StmError::Failure)?;
        assert_eq!(manager.read_attribute::<Temperature>(t, 3)?, None);
        assert_eq!(manager.read_attribute::<Temperature>(t, 6)?, None);
        assert_eq!(
            manager.read_attribute(t, 8)?,
            Some(Temperature::from(282.0))
        );
        Ok(())
    });
}

#[test]
fn manager_merge_undefined_attribute() {
    generate_manager!(manager);
    atomically(|t| {
        assert_eq!(
            manager.remove_attribute(t, 3)?,
            Some(Temperature::from(279.0))
        );
        assert_eq!(
            manager.remove_attribute(t, 6)?,
            Some(Temperature::from(285.0))
        );
        assert_eq!(
            manager.remove_attribute(t, 8)?,
            Some(Temperature::from(289.0))
        );
        // merge from two undefined value
        manager
            .merge_attribute::<Temperature>(t, 8, 3, 6)
            .map_err(|_| StmError::Failure)?;
        assert_eq!(manager.read_attribute::<Temperature>(t, 3)?, None);
        assert_eq!(manager.read_attribute::<Temperature>(t, 6)?, None);
        assert_eq!(manager.read_attribute(t, 8)?, Some(Temperature::from(0.0)));
        // merge from one undefined value
        assert_eq!(
            manager.read_attribute(t, 4)?,
            Some(Temperature::from(281.0))
        );
        manager
            .merge_attribute::<Temperature>(t, 6, 3, 4)
            .map_err(|_| StmError::Failure)?;
        assert_eq!(manager.read_attribute::<Temperature>(t, 3)?, None);
        assert_eq!(manager.read_attribute::<Temperature>(t, 4)?, None);
        assert_eq!(
            manager.read_attribute(t, 6)?,
            Some(Temperature::from(281.0 / 2.0))
        );
        Ok(())
    });
}

#[test]
fn manager_split_attribute() {
    generate_manager!(manager);
    atomically(|t| {
        assert_eq!(
            manager.remove_attribute(t, 3)?,
            Some(Temperature::from(279.0))
        );
        assert_eq!(
            manager.remove_attribute(t, 6)?,
            Some(Temperature::from(285.0))
        );
        assert_eq!(
            manager.read_attribute(t, 8)?,
            Some(Temperature::from(289.0))
        );
        manager
            .split_attribute::<Temperature>(t, 3, 6, 8)
            .map_err(|_| StmError::Failure)?;
        assert_eq!(
            manager.read_attribute(t, 3)?,
            Some(Temperature::from(289.0))
        );
        assert_eq!(
            manager.read_attribute(t, 6)?,
            Some(Temperature::from(289.0))
        );
        assert_eq!(manager.read_attribute::<Temperature>(t, 8)?, None);
        Ok(())
    });
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

        atomically(|t| {
            manager.write_attribute(t, 1, Temperature::from(20.0))?;
            manager.write_attribute(t, 3, Temperature::from(30.0))?;

            manager.write_attribute(t, 1, Length(3.0))?;
            manager.write_attribute(t, 3, Length(2.0))?;

            manager.write_attribute(t, 1, Weight(10))?;
            manager.write_attribute(t, 3, Weight(15))?;

            manager.write_attribute(t, 1, Color(255, 0, 0))?;
            manager.write_attribute(t, 3, Color(0, 0, 255))?;
            Ok(())
        });

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
                c1.merge_attributes(trans, OrbitPolicy::Vertex, 2, 1, 3)
                    .map_err(|_| StmError::Retry)?;
                c1.merge_attributes(trans, OrbitPolicy::Edge, 2, 1, 3)
                    .map_err(|_| StmError::Retry)?;
                c1.merge_attributes(trans, OrbitPolicy::Face, 2, 1, 3)
                    .map_err(|_| StmError::Retry)?;
                Ok(())
            });
        });

        let t2 = loom::thread::spawn(move || {
            atomically(|trans| {
                c2.split_attributes(trans, OrbitPolicy::Vertex, 2, 3, 2)
                    .map_err(|_| StmError::Retry)?;
                c2.split_attributes(trans, OrbitPolicy::Edge, 2, 3, 2)
                    .map_err(|_| StmError::Retry)?;
                c2.split_attributes(trans, OrbitPolicy::Face, 2, 3, 2)
                    .map_err(|_| StmError::Retry)?;
                Ok(())
            });
        });

        t1.join().unwrap();
        t2.join().unwrap();

        // in both cases
        let slot_1_is_empty = atomically(|t| {
            Ok(arc.read_attribute::<Temperature>(t, 1)?.is_none()
                && arc.read_attribute::<Weight>(t, 1)?.is_none()
                && arc.read_attribute::<Length>(t, 1)?.is_none()
                && arc.read_attribute::<Color>(t, 1)?.is_none())
        });
        assert!(slot_1_is_empty);

        let p1 = atomically(|t| {
            // path 1: merge before split
            let p1_2_temp = arc
                .read_attribute::<Temperature>(t, 2)?
                .is_some_and(|val| val == Temperature::from(25.0));
            let p1_3_temp = arc
                .read_attribute::<Temperature>(t, 3)?
                .is_some_and(|val| val == Temperature::from(25.0));

            let p1_2_weight = arc
                .read_attribute::<Weight>(t, 2)?
                .is_some_and(|v| v == Weight(13));
            let p1_3_weight = arc
                .read_attribute::<Weight>(t, 3)?
                .is_some_and(|v| v == Weight(12));

            let p1_2_len = arc
                .read_attribute::<Length>(t, 2)?
                .is_some_and(|v| v == Length(2.5));
            let p1_3_len = arc
                .read_attribute::<Length>(t, 3)?
                .is_some_and(|v| v == Length(2.5));

            let p1_2_col = arc
                .read_attribute::<Color>(t, 2)?
                .is_some_and(|v| v == Color(127, 0, 127));
            let p1_3_col = arc
                .read_attribute::<Color>(t, 3)?
                .is_some_and(|v| v == Color(127, 0, 127));

            Ok(slot_1_is_empty
                && p1_2_temp
                && p1_3_temp
                && p1_2_weight
                && p1_3_weight
                && p1_2_len
                && p1_3_len
                && p1_2_col
                && p1_3_col)
        });

        let p2 = atomically(|t| {
            // path 2: split before merge
            let p2_2_temp = arc
                .read_attribute::<Temperature>(t, 2)?
                .is_some_and(|val| val == Temperature::from(5.0));
            let p2_3_temp = arc.read_attribute::<Temperature>(t, 3)?.is_none();

            let p2_2_weight = arc
                .read_attribute::<Weight>(t, 2)?
                .is_some_and(|v| v == Weight(10));
            let p2_3_weight = arc.read_attribute::<Weight>(t, 3)?.is_none();

            let p2_2_len = arc
                .read_attribute::<Length>(t, 2)?
                .is_some_and(|v| v == Length(3.0));
            let p2_3_len = arc.read_attribute::<Length>(t, 3)?.is_none();

            let p2_2_col = arc
                .read_attribute::<Color>(t, 2)?
                .is_some_and(|v| v == Color(255, 0, 0));
            let p2_3_col = arc.read_attribute::<Color>(t, 3)?.is_none();

            Ok(slot_1_is_empty
                && p2_2_temp
                && p2_3_temp
                && p2_2_weight
                && p2_3_weight
                && p2_2_len
                && p2_3_len
                && p2_2_col
                && p2_3_col)
        });

        assert!(p1 || p2);
    });
}
