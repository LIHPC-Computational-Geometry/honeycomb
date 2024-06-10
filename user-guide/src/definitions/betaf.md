# Beta Functions

Each combinatorial map of dimension *N* defines *N* beta functions linking the set of darts together (e.g. a 2-map
contains *β<sub>1</sub>* and *β<sub>2</sub>*). These functions model the topology of the map, giving information about
connections of the different cells of the map / mesh. In our case, we mostly use:

- *β<sub>1</sub>*, a (partial) permutation,
- *β<sub>2</sub>*, *β<sub>3</sub>*, two (partial) involutions

Additionally, we define *β<sub>0</sub>* as the inverse of *β<sub>1</sub>*, i.e. *β<sub>0</sub>(β<sub>1</sub>(d)) = d*.
This comes from a practical consideration for performances and efficiency of the implementation.

The *β<sub>i</sub>* functions can be interpreted as navigation functions along the *i-th* dimensions: *β<sub>1</sub>*
makes you navigate along the edges, *β<sub>2</sub>* along the faces, etc. This can be generalized to *N* dimensions,
but we are only interested in 2D and 3D at the moment.

## Properties

For a given dart *d*, we define two properties:

- *d* is ***i*-free** if *β<sub>i</sub>(d) = ∅*, *∅* being the null dart
- *d* is **free** if it is ***i*-free for all *i***

## Construction Example

<figure style="text-align:center">
    <img src="../images/bg_darts.svg" alt="Embed" />
    <figcaption><i>Start from unorganized darts</i></figcaption>
</figure>

<figure style="text-align:center">
    <img src="../images/bg_beta1.svg" alt="Embed" />
    <figcaption><i>Organize those using β<sub>1</sub></i></figcaption>
</figure>

<figure style="text-align:center">
    <img src="../images/bg_beta2.svg" alt="Embed" />
    <figcaption><i>Add β<sub>2</sub> images; For details on vertices, refer to the Embedding section</i></figcaption>
</figure>

<figure style="text-align:center">
    <img src="../images/bg_map.svg" alt="Embed" />
    <figcaption><i>Build up a larger map</i></figcaption>
</figure>