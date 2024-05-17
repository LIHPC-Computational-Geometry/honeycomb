# Embedding

Embedding, or embedded data refers to the association of topological
entities (darts, *i*-cells) to geometrical data (spatial positions,
vertices, faces, volumes).

We choose to encode the origin of darts as their associated vertex. In
order to avoid duplicating coordinates, what is associated to each dart
is an identifier, meaning that all darts starting from a given point in 
space share the same associated vertex ID.

<figure style="text-align:center">
    <img src="../images/Embed.svg" alt="Embed" />
    <figcaption><i>Geometric embedding of spatial data</i></figcaption>
</figure>

In the above example, the data would be organized in the following way:

- *darts = { null, d1, d2, d3, d4 }*
- *associated_vertex = { null, v1, v2, v3, v4 }*
- *vertices = { (0.0, 0.0), (0.0, 1.0), (1.0, 1.0), (1.0, 0.0) }*

In this case, *v1 = 0*; *v2 = 1*; *v3 = 2*; *v4 = 3*.

Association of darts and vertices ID is done implicitly through indexing;
In practice, the *darts* vector does not even exist. This example is limited
to vertices, but we also keep track of faces, and volumes eventually.

The embedding of geometrical data also has implication for operations
on the map. This is detailed along operation specificities in their 
dedicated sections.