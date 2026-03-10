# Element insertion and deletion

**This content has been copy-pasted from the previous guide. It is up-to-date but should be improved
at some point.**

---

## Dart insertion

As explained in the [Darts](darts.html) section of this guide, these entities exist implicitly
through indexing of the internal storage structures of the map. Because of this, adding darts
translates to extending internal vectors and storages in our implementation.

An internal counter is incremented at each dart addition. This, coupled with an unused dart
tracking mechanism, constitutes a way to keep track of attributed darts.

## Dart deletion

Removing a dart would technically require us to remove an entry inside storage structures, which
are often ordered, contiguous vectors. There are two way to approach this problem:

- Actually remove the entry
    - requires adjustments on all the structure to keep consistent indices
    - keeps the storage compact, i.e. all allocated slots are used
- "Forget" the entry
    - does not require any re-arrangements besides making sure no beta functions lands on the dart
    - creates "holes" in the storage

Our implementation uses the second solution, along with a structure used to store unused slots.
In turns, we can use these "holes" in the storage to reinsert darts or collapse the structure at
a later point during execution.

## Add / Remove a dimension

Adding or removing a dimension on a given combinatorial maps effectively corresponds, respectively,
to adding or removing a beta function. In the case of decreasing the dimension, this operation can
result in two disjoint dart set in the same map.

Because the current implementation only covers 2D combinatiorial maps, this operation is not
implemented. When 3D maps are implemented, it would be possible to implement this using the
`From` trait provided by the Rust language.
