# Darts

Darts are the finest grain composing combinatorial maps. The structure
of the map is given by the relationship between darts, defined through 
beta functions.

<figure style="text-align:center">
    <img src="../images/Darts.svg" alt="Darts" />
    <figcaption><i>Boundless square modeling using four darts</i></figcaption>
</figure>

In our implementation, darts exist implictly through indexing and their
associated data. There are no dart *objects* in a strict sense, there is
only a given number of dart, their associated data ordered by an array-like
logic, and a record of "unused" slots that can be used for dart insertion.
Because of this, we assimilate dart and dart index.