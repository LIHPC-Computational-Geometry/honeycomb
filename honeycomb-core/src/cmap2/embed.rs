//! Attribute operations implementation
//!
//! This module contains code used to implement operations on the embedded data associated to the
//! map. This includes operations regarding vertices as well as (in the future) user-defined
//! generic attributes

// ------ IMPORT

use crate::{CMap2, CMapError, CoordsFloat, Vertex2, VertexIdentifier};

// ------ CONTENT

// --- vertex attributes
impl<T: CoordsFloat> CMap2<T> {
    /// Return the current number of vertices.
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn n_vertices(&self) -> usize {
        self.vertices.n_attributes()
    }

    /// Fetch vertex value associated to a given identifier.
    ///
    /// # Arguments
    ///
    /// - `vertex_id: VertexIdentifier` -- Identifier of the given vertex.
    ///
    /// # Return
    ///
    /// Return a reference to the [Vertex2] associated to the ID.
    ///
    /// # Panics
    ///
    /// The method may panic if no vertex is associated to the specified index, or the ID lands
    /// out of bounds.
    ///
    #[must_use = "returned value is not used, consider removing this method call"]
    pub fn vertex(&self, vertex_id: VertexIdentifier) -> Vertex2<T> {
        self.vertices.get(&vertex_id).unwrap()
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
    /// # Return
    ///
    /// Return an option which may contain the previous value associated to the specified vertex ID.
    ///
    pub fn insert_vertex(&mut self, vertex_id: VertexIdentifier, vertex: impl Into<Vertex2<T>>) {
        self.vertices.insert(&vertex_id, vertex.into());
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
        if let Some(val) = self.vertices.remove(&vertex_id) {
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
    /// returned
    /// - `Err(CMapError::UnknownVertexID)` -- The vertex was not found in the internal storage
    ///
    pub fn replace_vertex(
        &mut self,
        vertex_id: VertexIdentifier,
        vertex: impl Into<Vertex2<T>>,
    ) -> Result<Vertex2<T>, CMapError> {
        if let Some(val) = self.vertices.replace(&vertex_id, vertex.into()) {
            return Ok(val);
        };
        Err(CMapError::UndefinedVertex)
    }
}