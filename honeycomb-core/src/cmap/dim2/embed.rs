//! Attribute operations implementation
//!
//! This module contains code used to implement operations on the embedded data associated to the
//! map. This includes operations regarding vertices as well as (in the future) user-defined
//! generic attributes

// ------ IMPORT

use stm::{StmResult, Transaction};

use crate::prelude::{AttributeBind, AttributeUpdate, CMap2, Vertex2, VertexIdType};
use crate::{
    attributes::{AttributeStorage, UnknownAttributeStorage},
    geometry::CoordsFloat,
};

// ------ CONTENT

/// **Built-in vertex-related methods**
impl<T: CoordsFloat> CMap2<T> {
    /// Return the current number of vertices.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_vertices(&self) -> usize {
        self.vertices.n_attributes()
    }

    #[allow(clippy::missing_errors_doc)]
    /// Read vertex associated to a given identifier.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the given vertex.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually,
    /// only used via the `?` operator.
    ///
    /// This method return a `Option` taking the following values:
    /// - `Some(v: Vertex2)` if there is a vertex associated to this ID.
    /// - `None` -- otherwise
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn read_vertex(
        &self,
        trans: &mut Transaction,
        vertex_id: VertexIdType,
    ) -> StmResult<Option<Vertex2<T>>> {
        self.vertices.read(trans, vertex_id)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Write a vertex to a given identifier, and return its old value.
    ///
    /// This method can be interpreted as giving a value to the vertex of a specific ID. Vertices
    /// implicitly exist through topology, but their spatial representation is not automatically
    /// created at first.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Vertex identifier to attribute a value to.
    /// - `vertex: impl Into<Vertex2>` -- Value used to create a [Vertex2] value.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually,
    /// only used via the `?` operator.
    ///
    /// The result contains an `Option` taking the following values:
    /// - `Some(v: Vertex2)` -- The old value was successfull replaced & returned
    /// - `None` -- The value was successfully set
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - **there is already a vertex associated to the specified index**
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    pub fn write_vertex(
        &self,
        trans: &mut Transaction,
        vertex_id: VertexIdType,
        vertex: impl Into<Vertex2<T>>,
    ) -> StmResult<Option<Vertex2<T>>> {
        self.vertices.write(trans, vertex_id, vertex.into())
    }

    #[allow(clippy::missing_errors_doc)]
    /// Remove vertex associated to a given identifier and return it.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the vertex to remove.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually,
    /// only used via the `?` operator.
    ///
    /// The result contains an `Option` taking the following values:
    /// - `Some(v: Vertex2)` -- The vertex was successfully removed & its value was returned
    /// - `None` -- The vertex was not found in the internal storage
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    pub fn remove_vertex(
        &self,
        trans: &mut Transaction,
        vertex_id: VertexIdType,
    ) -> StmResult<Option<Vertex2<T>>> {
        self.vertices.remove(trans, vertex_id)
    }

    #[must_use = "returned value is not used, consider removing this method call"]
    /// Read vertex associated to a given identifier.
    ///
    /// This variant is equivalent to `read_vertex`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_read_vertex(&self, vertex_id: VertexIdType) -> Option<Vertex2<T>> {
        self.vertices.force_read(vertex_id)
    }

    /// Write a vertex to a given identifier, and return its old value.
    ///
    /// This variant is equivalent to `write_vertex`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_write_vertex(
        &self,
        vertex_id: VertexIdType,
        vertex: impl Into<Vertex2<T>>,
    ) -> Option<Vertex2<T>> {
        self.vertices.force_write(vertex_id, vertex.into())
    }

    #[allow(clippy::must_use_candidate)]
    /// Remove vertex associated to a given identifier and return it.
    ///
    /// This variant is equivalent to `remove_vertex`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_remove_vertex(&self, vertex_id: VertexIdType) -> Option<Vertex2<T>> {
        self.vertices.force_remove(vertex_id)
    }
}

/// **Generic attribute-related methods**
impl<T: CoordsFloat> CMap2<T> {
    #[allow(clippy::missing_errors_doc)]
    /// Read a given attribute's value associated to a given identifier.
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind + AttributeUpdate` -- Attribute to read.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually,
    /// only used via the `?` operator.
    ///
    /// The result contains an `Option` taking the following values:
    /// - `Some(v: Vertex2)` -- The old value was successfull replaced & returned
    /// - `None` -- The value was successfully set
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    pub fn read_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
    ) -> StmResult<Option<A>> {
        self.attributes.read_attribute::<A>(trans, id)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Write a given attribute's value to a given identifier, and return its old value.
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    /// - `val: A` -- Attribute value.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind + AttributeUpdate` -- Attribute to edit.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually,
    /// only used via the `?` operator.
    ///
    /// The result contains an `Option` taking the following values:
    /// - `Some(v: Vertex2)` -- The old value was successfull replaced & returned
    /// - `None` -- The value was successfully set
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    pub fn write_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
        val: A,
    ) -> StmResult<Option<A>> {
        self.attributes.write_attribute::<A>(trans, id, val)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Remove a given attribute's value from the storage and return it.
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind + AttributeUpdate` -- Attribute to edit.
    ///
    /// # Return / Errors
    ///
    /// This method is meant to be called in a context where the returned `Result` is used to
    /// validate the transaction passed as argument. The result should not be processed manually,
    /// only used via the `?` operator.
    ///
    /// The result contains an `Option` taking the following values:
    /// - `Some(val: A)` -- The vertex was successfully removed & its value was returned
    /// - `None` -- The vertex was not found in the internal storage
    ///
    /// # Panics
    ///
    /// The method:
    /// - may panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    pub fn remove_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        trans: &mut Transaction,
        id: A::IdentifierType,
    ) -> StmResult<Option<A>> {
        self.attributes.remove_attribute::<A>(trans, id)
    }

    /// Read a given attribute's value associated to a given identifier.
    ///
    /// This variant is equivalent to `read_attribute`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_read_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        id: A::IdentifierType,
    ) -> Option<A> {
        self.attributes.force_read_attribute::<A>(id)
    }

    /// Write a given attribute's value to a given identifier, and return its old value.
    ///
    /// This variant is equivalent to `write_attribute`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_write_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        id: A::IdentifierType,
        val: A,
    ) -> Option<A> {
        self.attributes.force_write_attribute::<A>(id, val)
    }

    /// Remove a given attribute's value from the storage and return it.
    ///
    /// This variant is equivalent to `remove_attribute`, but internally uses a transaction that will be
    /// retried until validated.
    pub fn force_remove_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        id: A::IdentifierType,
    ) -> Option<A> {
        self.attributes.force_remove_attribute::<A>(id)
    }
    // --- big guns

    /// Add a new attribute storage to the map.
    ///
    /// This method is useful for kernels using a specific attribute to execute, or as return
    /// values.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind + AttributeUpdate` -- Attribute stored by the fetched storage.
    pub fn add_attribute_storage<A: AttributeBind + AttributeUpdate>(&mut self) {
        self.attributes.add_storage::<A>(self.n_darts() + 1);
    }

    /// Remove an entire attribute storage from the map.
    ///
    /// This method is useful when implementing routines that uses attributes to run; Those can then be removed
    /// before the final result is returned.
    ///
    /// # Generic
    ///
    /// - `A: AttributeBind + AttributeUpdate` -- Attribute stored by the fetched storage.
    pub fn remove_attribute_storage<A: AttributeBind + AttributeUpdate>(&mut self) {
        self.attributes.remove_storage::<A>();
    }
}
