# Add / Remove a free dart

## Adding darts

As explained in the [Darts](darts.html) section of this guide, these entities exist implictly through indexing of the
internal storage structures of the map. Because of this, adding darts translates to extending internal vectors and
storages in our implementation.

An internal counter is incremented at each dart addition. This, coupled with an unused dart tracking mechanism,
constitutes a way to keep track of attributed darts.

## Removing darts

Removing a dart would technically require us to remove an entry inside storage structures, which are often ordered,
contiguous vectors. There are two way to approach this problem:

- Actually remove the entry
    - requires adjustments on all the structure to keep consistent indices
    - keeps the storage compact, i.e. all allocated slots are used
- "Forget" the entry
    - does not require any re-arrangements besides making sure no beta functions lands on the dart
    - creates "holes" in the storage

Our implementation uses the second solution, along with a structure used to store unused slots. In turns, we can use
these "holes" in the storage to reinsert darts or collapse the structure at a later point during execution.
