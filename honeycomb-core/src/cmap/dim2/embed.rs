//! Attribute operations implementation
//!
//! This module contains code used to implement operations on the embedded data associated to the
//! map. This includes operations regarding vertices as well as (in the future) user-defined
//! generic attributes

use crate::attributes::{
    AttributeBind, AttributeError, AttributeStorage, AttributeUpdate, UnknownAttributeStorage,
};
use crate::cmap::components::access::AccessController;
use crate::cmap::{CMap2, DartIdType, OrbitPolicy, VertexIdType};
use crate::geometry::{CoordsFloat, Vertex2};
use crate::stm::{
    StmClosureResult, Transaction, TransactionClosureResult, TransactionError, atomically,
};

/// **Access-routed internals**
///
/// These methods route each access to the map's embedded data through either the transactional
/// machinery or direct atomic operations, depending on the map's access controller `AC`. They
/// are the building blocks used by all other operations of the structure, so that every access
/// is consistently regulated.
impl<T: CoordsFloat, AC: AccessController> CMap2<T, AC> {
    /// Route a vertex read according to `AC::VERTICES_TX_ACCESS`.
    pub(super) fn vertex_read(
        &self,
        t: &mut Transaction,
        vertex_id: VertexIdType,
    ) -> StmClosureResult<Option<Vertex2<T>>> {
        if AC::VERTICES_TX_ACCESS {
            self.vertices.read(t, vertex_id)
        } else {
            Ok(self.vertices.read_atomic(vertex_id))
        }
    }

    /// Route a vertex write according to `AC::VERTICES_TX_ACCESS`.
    pub(super) fn vertex_write(
        &self,
        t: &mut Transaction,
        vertex_id: VertexIdType,
        vertex: Vertex2<T>,
    ) -> StmClosureResult<Option<Vertex2<T>>> {
        if AC::VERTICES_TX_ACCESS {
            self.vertices.write(t, vertex_id, vertex)
        } else {
            Ok(self.vertices.write_atomic(vertex_id, vertex))
        }
    }

    /// Route a vertex removal according to `AC::VERTICES_TX_ACCESS`.
    pub(super) fn vertex_remove(
        &self,
        t: &mut Transaction,
        vertex_id: VertexIdType,
    ) -> StmClosureResult<Option<Vertex2<T>>> {
        if AC::VERTICES_TX_ACCESS {
            self.vertices.remove(t, vertex_id)
        } else {
            Ok(self.vertices.remove_atomic(vertex_id))
        }
    }

    /// Route a vertex merge according to `AC::VERTICES_TX_ACCESS`.
    pub(super) fn vertex_merge(
        &self,
        t: &mut Transaction,
        out: DartIdType,
        lhs_inp: DartIdType,
        rhs_inp: DartIdType,
    ) -> TransactionClosureResult<(), AttributeError> {
        if AC::VERTICES_TX_ACCESS {
            self.vertices.merge(t, out, lhs_inp, rhs_inp)
        } else {
            self.vertices
                .merge_atomic(out, lhs_inp, rhs_inp)
                .map_err(TransactionError::Abort)
        }
    }

    /// Route a vertex split according to `AC::VERTICES_TX_ACCESS`.
    pub(super) fn vertex_split(
        &self,
        t: &mut Transaction,
        lhs_out: DartIdType,
        rhs_out: DartIdType,
        inp: DartIdType,
    ) -> TransactionClosureResult<(), AttributeError> {
        if AC::VERTICES_TX_ACCESS {
            self.vertices.split(t, lhs_out, rhs_out, inp)
        } else {
            self.vertices
                .split_atomic(lhs_out, rhs_out, inp)
                .map_err(TransactionError::Abort)
        }
    }

    /// Route a user attribute merge according to `AC::ATTRIBUTES_TX_ACCESS`.
    pub(super) fn merge_attributes(
        &self,
        t: &mut Transaction,
        orbit_policy: OrbitPolicy,
        id_out: DartIdType,
        id_in_lhs: DartIdType,
        id_in_rhs: DartIdType,
    ) -> TransactionClosureResult<(), AttributeError> {
        if AC::ATTRIBUTES_TX_ACCESS {
            self.attributes
                .merge_attributes(t, orbit_policy, id_out, id_in_lhs, id_in_rhs)
        } else {
            self.attributes
                .merge_attributes_atomic(orbit_policy, id_out, id_in_lhs, id_in_rhs)
                .map_err(TransactionError::Abort)
        }
    }

    /// Route a user attribute split according to `AC::ATTRIBUTES_TX_ACCESS`.
    pub(super) fn split_attributes(
        &self,
        t: &mut Transaction,
        orbit_policy: OrbitPolicy,
        id_out_lhs: DartIdType,
        id_out_rhs: DartIdType,
        id_in: DartIdType,
    ) -> TransactionClosureResult<(), AttributeError> {
        if AC::ATTRIBUTES_TX_ACCESS {
            self.attributes
                .split_attributes(t, orbit_policy, id_out_lhs, id_out_rhs, id_in)
        } else {
            self.attributes
                .split_attributes_atomic(orbit_policy, id_out_lhs, id_out_rhs, id_in)
                .map_err(TransactionError::Abort)
        }
    }

    /// Route user attribute slot clears according to `AC::ATTRIBUTES_TX_ACCESS`.
    pub(super) fn clear_attribute_values(
        &self,
        t: &mut Transaction,
        id: DartIdType,
    ) -> StmClosureResult<()> {
        if AC::ATTRIBUTES_TX_ACCESS {
            self.attributes.clear_attribute_values(t, id)
        } else {
            self.attributes.clear_attribute_values_atomic(id);
            Ok(())
        }
    }
}

/// **Built-in vertex-related methods**
impl<T: CoordsFloat, AC: AccessController> CMap2<T, AC> {
    /// Return the current number of vertices.
    #[must_use = "unused return value"]
    pub fn n_vertices(&self) -> usize {
        self.vertices.n_attributes()
    }

    #[allow(clippy::missing_errors_doc)]
    /// Return the vertex associated to a given identifier.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// This method return a `Option` taking the following values:
    /// - `Some(v: Vertex2)` if there is a vertex associated to this ID,
    /// - `None` otherwise.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds,
    /// - the index cannot be converted to `usize`.
    pub fn read_vertex_tx(
        &self,
        t: &mut Transaction,
        vertex_id: VertexIdType,
    ) -> StmClosureResult<Option<Vertex2<T>>> {
        self.vertex_read(t, vertex_id)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Replace the vertex associated to a given identifier and return its old value.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the vertex to replace.
    /// - `vertex: impl Into<Vertex2>` -- New [`Vertex2`] value.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// The result contains an `Option` taking the following values:
    /// - `Some(v: Vertex2)` if there was an old value,
    /// - `None` otherwise.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds,
    /// - the index cannot be converted to `usize`.
    pub fn write_vertex_tx(
        &self,
        t: &mut Transaction,
        vertex_id: VertexIdType,
        vertex: impl Into<Vertex2<T>>,
    ) -> StmClosureResult<Option<Vertex2<T>>> {
        self.vertex_write(t, vertex_id, vertex.into())
    }

    #[allow(clippy::missing_errors_doc)]
    /// Remove the vertex associated to a given identifier and return it.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    //     /// only processed via the `?` operator.
    ///
    /// The result contains an `Option` taking the following values:
    /// - `Some(v: Vertex2)` if there was a value,
    /// - `None` otherwise.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds,
    /// - the index cannot be converted to `usize`.
    pub fn remove_vertex_tx(
        &self,
        t: &mut Transaction,
        vertex_id: VertexIdType,
    ) -> StmClosureResult<Option<Vertex2<T>>> {
        self.vertex_remove(t, vertex_id)
    }

    #[must_use = "unused return value"]
    /// Read the vertex associated to a given identifier.
    ///
    /// This variant is equivalent to `read_vertex`, but internally uses a transaction that will be
    /// retried until validated. If the map's access controller regulates vertices atomically, no
    /// transaction is created and the value is read directly.
    pub fn read_vertex(&self, vertex_id: VertexIdType) -> Option<Vertex2<T>> {
        if AC::VERTICES_TX_ACCESS {
            atomically(|t| self.vertices.read(t, vertex_id))
        } else {
            self.vertices.read_atomic(vertex_id)
        }
    }

    /// Replace the vertex associated to a given identifier and return its old value.
    ///
    /// This variant is equivalent to `write_vertex`, but internally uses a transaction that will be
    /// retried until validated. If the map's access controller regulates vertices atomically, no
    /// transaction is created and the value is written directly.
    pub fn write_vertex(
        &self,
        vertex_id: VertexIdType,
        vertex: impl Into<Vertex2<T>>,
    ) -> Option<Vertex2<T>> {
        let tmp = vertex.into();
        if AC::VERTICES_TX_ACCESS {
            atomically(|t| self.vertices.write(t, vertex_id, tmp))
        } else {
            self.vertices.write_atomic(vertex_id, tmp)
        }
    }

    #[allow(clippy::must_use_candidate)]
    /// Remove the vertex associated to a given identifier and return it.
    ///
    /// This variant is equivalent to `remove_vertex`, but internally uses a transaction that will
    /// be retried until validated. If the map's access controller regulates vertices atomically,
    /// no transaction is created and the value is removed directly.
    pub fn remove_vertex(&self, vertex_id: VertexIdType) -> Option<Vertex2<T>> {
        if AC::VERTICES_TX_ACCESS {
            atomically(|t| self.vertices.remove(t, vertex_id))
        } else {
            self.vertices.remove_atomic(vertex_id)
        }
    }
}

/// **Generic attribute-related methods**
impl<T: CoordsFloat, AC: AccessController> CMap2<T, AC> {
    #[allow(clippy::missing_errors_doc)]
    /// Return the attribute `A` value associated to a given identifier.
    ///
    /// The kind of cell `A` binds to is automatically deduced using its `AttributeBind`
    /// implementation.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// This method return a `Option` taking the following values:
    /// - `Some(a: A)` if there is a value associated to this ID,
    /// - `None` otherwise, or if there is no storage for this kind of attribute in the map.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds,
    /// - the index cannot be converted to `usize`.
    pub fn read_attribute_tx<A: AttributeBind + AttributeUpdate>(
        &self,
        t: &mut Transaction,
        id: A::IdentifierType,
    ) -> StmClosureResult<Option<A>> {
        if AC::ATTRIBUTES_TX_ACCESS {
            self.attributes.read_attribute::<A>(t, id)
        } else {
            Ok(self.attributes.read_attribute_atomic::<A>(id))
        }
    }

    #[allow(clippy::missing_errors_doc)]
    /// Replace the attribute `A` value associated to a given identifier and return its old value.
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Identifier of the cell's value to replace.
    /// - `val: A` -- Attribute value.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// The result contains an `Option` taking the following values:
    /// - `Some(a: A)` if there was an old value,
    /// - `None` otherwise, or if there is no storage for this kind of attribute in the map.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds,
    /// - the index cannot be converted to `usize`.
    pub fn write_attribute_tx<A: AttributeBind + AttributeUpdate>(
        &self,
        t: &mut Transaction,
        id: A::IdentifierType,
        val: A,
    ) -> StmClosureResult<Option<A>> {
        if AC::ATTRIBUTES_TX_ACCESS {
            self.attributes.write_attribute::<A>(t, id, val)
        } else {
            Ok(self.attributes.write_attribute_atomic::<A>(id, val))
        }
    }

    #[allow(clippy::missing_errors_doc)]
    /// Remove the attribute `A` value associated to a given identifier and return it.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. Errors should not be processed manually,
    /// only processed via the `?` operator.
    ///
    /// The result contains an `Option` taking the following values:
    /// - `Some(a: A)` if there was a value,
    /// - `None` otherwise, or if there is no storage for this kind of attribute in the map.
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds,
    /// - the index cannot be converted to `usize`.
    pub fn remove_attribute_tx<A: AttributeBind + AttributeUpdate>(
        &self,
        t: &mut Transaction,
        id: A::IdentifierType,
    ) -> StmClosureResult<Option<A>> {
        if AC::ATTRIBUTES_TX_ACCESS {
            self.attributes.remove_attribute::<A>(t, id)
        } else {
            Ok(self.attributes.remove_attribute_atomic::<A>(id))
        }
    }

    /// Return the attribute `A` value associated to a given identifier.
    ///
    /// This variant is equivalent to `read_attribute`, but internally uses a transaction that will be
    /// retried until validated. If the map's access controller regulates user attributes
    /// atomically, no transaction is created and the value is read directly.
    #[allow(clippy::needless_pass_by_value)]
    pub fn read_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        id: A::IdentifierType,
    ) -> Option<A> {
        if AC::ATTRIBUTES_TX_ACCESS {
            atomically(|t| self.attributes.read_attribute::<A>(t, id.clone()))
        } else {
            self.attributes.read_attribute_atomic::<A>(id)
        }
    }

    /// Replace the attribute `A` value associated to a given identifier and return its old value.
    ///
    /// This variant is equivalent to `write_attribute`, but internally uses a transaction that will be
    /// retried until validated. If the map's access controller regulates user attributes
    /// atomically, no transaction is created and the value is written directly.
    #[allow(clippy::needless_pass_by_value)]
    pub fn write_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        id: A::IdentifierType,
        val: A,
    ) -> Option<A> {
        if AC::ATTRIBUTES_TX_ACCESS {
            atomically(|t| self.attributes.write_attribute::<A>(t, id.clone(), val))
        } else {
            self.attributes.write_attribute_atomic::<A>(id, val)
        }
    }

    /// Remove the attribute `A` value associated to a given identifier and return it.
    ///
    /// This variant is equivalent to `remove_attribute`, but internally uses a transaction that
    /// will be retried until validated. If the map's access controller regulates user attributes
    /// atomically, no transaction is created and the value is removed directly.
    #[allow(clippy::needless_pass_by_value)]
    pub fn remove_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        id: A::IdentifierType,
    ) -> Option<A> {
        if AC::ATTRIBUTES_TX_ACCESS {
            atomically(|t| self.attributes.remove_attribute::<A>(t, id.clone()))
        } else {
            self.attributes.remove_attribute_atomic::<A>(id)
        }
    }

    // --- big guns

    /// Remove the attribute `A`'s storage from the map.
    ///
    /// This method is useful when implementing routines that uses attributes to run; Those can
    /// then be removed before the final result is returned.
    pub fn remove_attribute_storage<A: AttributeBind + AttributeUpdate>(&mut self) {
        self.attributes.remove_storage::<A>();
    }

    /// Return a boolean indicating if the map contains the specified attribute.
    #[must_use = "unused return value"]
    pub fn contains_attribute<A: AttributeBind + AttributeUpdate>(&self) -> bool {
        self.attributes.contains_attribute::<A>()
    }
}
