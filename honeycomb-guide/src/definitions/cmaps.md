# Combinatorial Maps

N-dimensional combinatorial maps, noted *N-map*, are objects made up of
two main elements:

- A set of darts, darts being the smallest elements making up the map
- N beta functions, linking the darts of the map

Additionally, we can define *embedded data* as spatial anchoring of the
darts making up the map. While the first two elements hold topological
information, embedded data gives information about the "shape" of the
map (e.g. vertices position in a spatial domain).

With these elements, we can represent and operate on meshes.

## Example

Operations on a combinatorial map can affect its topology, shape or both:

<figure style="text-align:center">
    <img src="../images/CMap2Mesh.svg" alt="MapMeshEquivalent" />
    <figcaption><i>Mesh-Map equivalent of a four step transformation</i></figcaption>
</figure>

The specifics on how data is encoded is detailed in attribute-specific
sections.