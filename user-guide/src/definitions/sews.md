# Sewing operation

---

Sew and unsew operations update the beta function values to modify the topological relation between
two or more darts. An \\(i\\)-dimensional sew can be interpreted as creating a connection between
two \\(i\\)-dimensional cell. That connection takes the form of an adjacency, and the definition of
the operation ensures that local structure remains consistent (in particular,  cells incident to
the new adjacency).

## Sewing

The sew operation can be divided into two parts:

- a topological update, which corresponds to a \\(\beta\\) function update to model a new
  topological relation
- a geometrical update, which corresponds to an update of the affected embedded data (attributes)

We call \\(i\\)-link the sub-operation corresponding to the topological update; Our implementation
provide it along with sews due to performance and flexibility concerns.

### Topology

The \\(i\\)-link operation corresponds to the aforementioned topological update. Given two darts
\\(d_a\\) and \\(d_b\\), and a given beta function \\(\beta_i\\), a link operation
corresponds to the update of the \\(\beta_i\\) function in order to have
... 


<figure style="text-align:center">
    <img src="../images/1sew.svg" alt="OneSew" width=70%/>
    <figcaption><i>1-sew between d<sub>1</sub> and d<sub>4</sub>.</i></figcaption>
</figure>

<figure style="text-align:center">
    <img src="../images/2sew.svg" alt="TwoSew" width=100%/>
    <figcaption><i>2-sew between d<sub>2</sub> and d<sub>5</sub>.</i></figcaption>
</figure>

Exact properties of the link operation directly depends on the property
of the modified beta function.

<figure style="text-align:center">
    <img src="../images/3sewable.svg" alt="ThreeSewable" width=80%/>
    <figcaption><i>Example of non 3-sewable (left) and 3-sewable (right) orbits.</i></figcaption>
</figure>

### Geometry

<figure style="text-align:center">
    <img src="../images/embedding-sew.png" alt="OneSew" width=100%/>
    <figcaption><i>Effect of 2-sew on i-cell composition.</i></figcaption>
</figure>

The *i-sew* operation corresponds to an *i-link* operation, coupled with an update of the affected
attributes. *How* the attributes are updated is defined through trait implementation in the Rust
crate (see [AttributeUpdate](../../honeycomb_core/attributes/trait.AttributeUpdate.html),
[AttributeBind](../../honeycomb_core/attributes/trait.AttributeBind.html)). *Which* attributes are
updated can be deduced from the dimension *i* of the sewing operation. This is summarized in
the following table:

| Dimension | Geometrical operation | 0-cell / Vertex Attributes | 1-cell / Edge Attributes | 2-cell / Face Attributes | 3-cell / Volume Attributes |
|-----------|-----------------------|----------------------------|--------------------------|--------------------------|----------------------------|
| 1         | Fusing vertices       | affected                   | unaffected               | unaffected               | unaffected                 |
| 2         | Fusing edges          | affected                   | affected                 | unaffected               | unaffected                 |
| 3         | Fusing faces          | affected                   | affected                 | affected                 | unaffected                 |

## Unsewing

The unsew operation is the complementary to the sew operation. It behaves according to similar
properties, but is used to remove links between darts. It does so by replacing values of the beta
functions by the null dart. Geometrical updates are handled and defined in the same way as for the
sew operation.
