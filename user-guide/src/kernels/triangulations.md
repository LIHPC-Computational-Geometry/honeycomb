# Polygon triangulation

---

We implement two functions for polygon triangulation. These are not meshing functions; our goal with
these is to cut existing cells of an irregular mesh into triangular cells.

With consideration to the above, we implement two polygon triangulation methods: *fanning*, and *ear-clipping*.
Both implementations are designed to operate in parallel; they take pre-allocated darts as argument and do not
create any contention over data as long as two calls are not made on the same face (2-cell).


## Fanning

Two versions of this algorithm are implemented:

The first implementation is a defensive one where the function actively search for a valid vertex to fan from.

The second implementation  assume the cell is convex; it fans the polygon from its first vertex. Convexity is not
checked, so use this only if you know all your cells fit the requirements!


## Ear-clipping

This method isn't algorithmically efficient, but we operate on small cells, and it covers our needs: it is a potential
fallback for non-fannable polygons without holes.

