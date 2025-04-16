//! Attribute operations implementation
//!
//! This module contains code used to implement operations on the embedded data associated to the
//! map. This includes operations regarding vertices as well as (in the future) user-defined
//! generic attributes

use crate::cmap::{CMap2, VertexIdType};
use crate::stm::{StmClosureResult, Transaction, atomically};
use crate::{
    attributes::{AttributeBind, AttributeStorage, AttributeUpdate, UnknownAttributeStorage},
    geometry::{CoordsFloat, Vertex2},
};

/// **Built-in vertex-related methods**
impl<T: CoordsFloat> CMap2<T> {
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
    pub fn read_vertex(
        &self,
        trans: &mut Transaction,
        vertex_id: VertexIdType,
    ) -> StmClosureResult<Option<Vertex2<T>>> {
        self.vertices.read(trans, vertex_id)
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
    pub fn write_vertex(
        &self,
        trans: &mut Transaction,
        vertex_id: VertexIdType,
        vertex: impl Into<Vertex2<T>>,
    ) -> StmClosureResult<Option<Vertex2<T>>> {
        self.vertices.write(trans, vertex_id, vertex.into())
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
    pub fn remove_vertex(
        &self,
        trans: &mut Transaction,
        vertex_id: VertexIdType,
    ) -> StmClosureResult<Option<Vertex2<T>>> {
        self.vertices.remove(trans, vertex_id)
    }

    #[must_use = "unused return value"]
    /// Read the vertex associated to a given identifier.
    ///
    /// This variant is equivalent to `read_vertex`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_read_vertex(&self, vertex_id: VertexIdType) -> Option<Vertex2<T>> {
        atomically(|t| self.vertices.read(t, vertex_id))
    }

    /// Replace the vertex associated to a given identifier and return its old value.
    ///
    /// This variant is equivalent to `write_vertex`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_write_vertex(
        &self,
        vertex_id: VertexIdType,
        vertex: impl Into<Vertex2<T>>,
    ) -> Option<Vertex2<T>> {
        let tmp = vertex.into();
        atomically(|t| self.vertices.write(t, vertex_id, tmp))
    }

    #[allow(clippy::must_use_candidate)]
    /// Remove the vertex associated to a given identifier and return it.
    ///
    /// This variant is equivalent to `remove_vertex`, but internally uses a transaction that will
    /// be retried until validated.
    pub fn force_remove_vertex(&self, vertex_id: VertexIdType) -> Option<Vertex2<T>> {
        atomically(|t| self.vertices.remove(t, vertex_id))
    }
}

/// **Generic attribute-related methods**
impl<T: CoordsFloat> CMap2<T> {
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
    pub fn read_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
    ) -> StmClosureResult<Option<A>> {
        self.attributes.read_attribute::<A>(trans, id)
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
    pub fn write_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
        val: A,
    ) -> StmClosureResult<Option<A>> {
        self.attributes.write_attribute::<A>(trans, id, val)
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
    pub fn remove_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
    ) -> StmClosureResult<Option<A>> {
        self.attributes.remove_attribute::<A>(trans, id)
    }

    /// Return the attribute `A` value associated to a given identifier.
    ///
    /// This variant is equivalent to `read_attribute`, but internally uses a transaction that will be
    /// retried until validated.
    #[allow(clippy::needless_pass_by_value)]
    pub fn force_read_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        id: A::IdentifierType,
    ) -> Option<A> {
        atomically(|t| self.attributes.read_attribute::<A>(t, id.clone()))
    }

    /// Replace the attribute `A` value associated to a given identifier and return its old value.
    ///
    /// This variant is equivalent to `write_attribute`, but internally uses a transaction that will be
    /// retried until validated.
    #[allow(clippy::needless_pass_by_value)]
    pub fn force_write_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        id: A::IdentifierType,
        val: A,
    ) -> Option<A> {
        atomically(|t| self.attributes.write_attribute::<A>(t, id.clone(), val))
    }

    /// Remove the attribute `A` value associated to a given identifier and return it.
    ///
    /// This variant is equivalent to `remove_attribute`, but internally uses a transaction that
    /// will be retried until validated.
    #[allow(clippy::needless_pass_by_value)]
    pub fn force_remove_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        id: A::IdentifierType,
    ) -> Option<A> {
        atomically(|t| self.attributes.remove_attribute::<A>(t, id.clone()))
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
