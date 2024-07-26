//! Attribute operations implementation
//!
//! This module contains code used to implement operations on the embedded data associated to the
//! map. This includes operations regarding vertices as well as (in the future) user-defined
//! generic attributes

// ------ IMPORT

use crate::{
    AttributeBind, AttributeStorage, AttributeUpdate, CMap2, CMapError, CoordsFloat,
    UnknownAttributeStorage, Vertex2, VertexIdentifier,
};

// ------ CONTENT

// --- big guns

impl<T: CoordsFloat> CMap2<T> {
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

// --- vertex attributes
impl<T: CoordsFloat> CMap2<T> {
    /// Return the current number of vertices.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_vertices(&self) -> usize {
        self.vertices.n_attributes()
    }

    #[allow(clippy::missing_errors_doc)]
    /// Fetch vertex value associated to a given identifier.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the given vertex.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(v: Vertex2)` if there is a vertex associated to this ID.
    /// - `Err(CMapError::UndefinedVertexID)` -- otherwise
    ///
    /// # Panics
    ///
    /// The method may panic if:
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn vertex(&self, vertex_id: VertexIdentifier) -> Result<Vertex2<T>, CMapError> {
        if let Some(val) = self.vertices.get(vertex_id) {
            return Ok(val);
        }
        Err(CMapError::UndefinedVertex)
    }

    /// Insert a vertex in the combinatorial map.
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
    /// # Panics
    ///
    /// The method may panic if:
    /// - **there is already a vertex associated to the specified index**
    /// - the index lands out of bounds
    /// - the index cannot be converted to `usize`
    ///
    pub fn insert_vertex(&mut self, vertex_id: VertexIdentifier, vertex: impl Into<Vertex2<T>>) {
        self.vertices.insert(vertex_id, vertex.into());
    }

    #[allow(clippy::missing_errors_doc)]
    /// Remove a vertex from the combinatorial map.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the vertex to remove.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(v: Vertex2)` -- The vertex was successfully removed & its value was returned
    /// - `Err(CMapError::UndefinedVertexID)` -- The vertex was not found in the internal storage
    ///
    pub fn remove_vertex(&mut self, vertex_id: VertexIdentifier) -> Result<Vertex2<T>, CMapError> {
        if let Some(val) = self.vertices.remove(vertex_id) {
            return Ok(val);
        }
        Err(CMapError::UndefinedVertex)
    }

    #[allow(clippy::missing_errors_doc)]
    /// Try to overwrite the given vertex with a new value.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the vertex to replace.
    /// - `vertex: impl<Into<Vertex2>>` -- New value for the vertex.
    ///
    /// # Return / Errors
    ///
    /// This method return a `Result` taking the following values:
    /// - `Ok(v: Vertex2)` -- The vertex was successfully overwritten & its previous value was
    ///   returned
    /// - `Err(CMapError::UnknownVertexID)` -- The vertex was not found in the internal storage
    ///
    pub fn replace_vertex(
        &mut self,
        vertex_id: VertexIdentifier,
        vertex: impl Into<Vertex2<T>>,
    ) -> Result<Vertex2<T>, CMapError> {
        if let Some(val) = self.vertices.replace(vertex_id, vertex.into()) {
            return Ok(val);
        };
        Err(CMapError::UndefinedVertex)
    }
}

// --- generic attributes

impl<T: CoordsFloat> CMap2<T> {
    /// Setter
    ///
    /// Set the value of an attribute for a given index. This operation is not affected by
    /// the initial state of the edited entry.
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    /// - `val: A` -- Attribute value.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind + AttributeUpdate` -- Attribute kind to edit.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    pub fn set_attribute<A: AttributeBind + AttributeUpdate>(
        &mut self,
        id: A::IdentifierType,
        val: A,
    ) {
        self.attributes.set_attribute::<A>(id, val);
    }

    /// Setter
    ///
    /// Insert an attribute value at a given undefined index. See [#Panics] section information
    /// on behavior if the value is already defined.
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    /// - `val: A` -- Attribute value.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind + AttributeUpdate` -- Attribute kind to edit.
    ///
    /// # Panics
    ///
    /// The method:
    /// - **should panic if there is already a value associated to the specified index**
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    pub fn insert_attribute<A: AttributeBind + AttributeUpdate>(
        &mut self,
        id: A::IdentifierType,
        val: A,
    ) {
        self.attributes.insert_attribute::<A>(id, val);
    }

    /// Getter
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind + AttributeUpdate` -- Attribute kind to edit.
    ///
    /// # Return
    ///
    /// The method should return:
    /// - `Some(val: A)` if there is an attribute associated with the specified index,
    /// - `None` if there is not.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    pub fn get_attribute<A: AttributeBind + AttributeUpdate>(
        &self,
        id: A::IdentifierType,
    ) -> Option<A> {
        self.attributes.get_attribute::<A>(id)
    }

    /// Setter
    ///
    /// Replace the value of the attribute for a given index.
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    /// - `val: A` -- Attribute value.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind + AttributeUpdate` -- Attribute kind to edit.
    ///
    /// # Return
    ///
    /// The method should return:
    /// - `Some(val_old: A)` if there was an attribute associated with the specified index,
    /// - `None` if there is not.
    ///
    /// In both cases, the new value should be set to the one specified as argument.
    ///
    /// # Panics
    ///
    /// The method:
    /// - should panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    pub fn replace_attribute<A: AttributeBind + AttributeUpdate>(
        &mut self,
        id: A::IdentifierType,
        val: A,
    ) -> Option<A> {
        self.attributes.replace_attribute::<A>(id, val)
    }

    /// Remove an attribute value from the storage and return it
    ///
    /// # Arguments
    ///
    /// - `index: A::IdentifierType` -- Cell index.
    ///
    /// ## Generic
    ///
    /// - `A: AttributeBind + AttributeUpdate` -- Attribute kind to edit.
    ///
    /// # Return
    ///
    /// The method should return:
    /// - `Some(val: A)` if there was an attribute associated with the specified index,
    /// - `None` if there is not.
    ///
    /// # Panics
    ///
    /// The method:
    /// - may panic if the index lands out of bounds
    /// - may panic if the index cannot be converted to `usize`
    pub fn remove_attribute<A: AttributeBind + AttributeUpdate>(
        &mut self,
        id: A::IdentifierType,
    ) -> Option<A> {
        self.attributes.remove_attribute::<A>(id)
    }
}
