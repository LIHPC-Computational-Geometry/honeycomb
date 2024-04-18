# Sewing operation

The sew operation can be divided into two parts:

- a topological update, which corresponds to linking the darts
- a geometrical update, which corresponds to an update of the affected embedded data, called attributes in our code

Note that the implementation is not as simple as doing one and then the other for consistency reasons: changing the
topology affects our ability to retrieve the embedded data, therefore the result is highly sensitive to operation
order.

## Linking

The *i-link* operation corresponds to the aforementionned topological update. Given two darts *d<sub>a</sub>* and
*d<sub>b</sub>*, and a given beta function *β<sub>i</sub>*, a link operation corresponds to the update of the
*β<sub>i</sub>* function in order to have *β<sub>i</sub>(d<sub>a</sub>) = d<sub>b</sub>* and/or
*β<sub>i</sub>(d<sub>b</sub>) = d<sub>a</sub>* depending on darts order and *i*. For example:

- *1-link(d<sub>a</sub>,d<sub>b</sub>)* results in:
    - *β<sub>1</sub>(d<sub>a</sub>) = d<sub>b</sub>*
    - **if *β<sub>0</sub>* is defined**, *β<sub>0</sub>(d<sub>b</sub>) = d<sub>a</sub>*
- *1-link(d<sub>b</sub>,d<sub>a</sub>)* results in:
    - *β<sub>1</sub>(d<sub>b</sub>) = d<sub>a</sub>*
    - **if *β<sub>0</sub>* is defined**, *β<sub>0</sub>(d<sub>a</sub>) = d<sub>b</sub>*
- *2-link(d<sub>a</sub>,d<sub>b</sub>)* results in:
    - *β<sub>2</sub>(d<sub>a</sub>) = d<sub>b</sub>*
    - *β<sub>2</sub>(d<sub>b</sub>) = d<sub>a</sub>*
- *2-link(d<sub>b</sub>,d<sub>a</sub>)* results in the same changes as *2-link(d<sub>a</sub>,d<sub>b</sub>)*

Exact properties of the link operation directly depends on the property of the modified beta function.

## Sewing

The *i-sew* operation corresponds to an *i-link* operation, coupled with an update of the affected attributes. *How*
the attributes are updated is defined through trait implementation in the Rust crate (see
[AttributeUpdate](../honeycomb_core/trait.AttributeUpdate.html),
[AttributeBind](../honeycomb_core/trait.AttributeBind.html)). *Which* attributes are updated can be deduced from the
dimension *i* of the sewing operation. This is summarized in the following table:

| Dimension | Geometrical operation | 0-cell / Vertex Attributes | 1-cell / Edge Attributes | 2-cell / Face Attributes | 3-cell / Volume Attributes |
|-----------|-----------------------|----------------------------|--------------------------|--------------------------|----------------------------|
| 1         | Fusing vertices       | affected                   | unaffected               | unaffected               | unaffected                 |
| 2         | Fusing edges          | affected                   | affected                 | unaffected               | unaffected                 |
| 3         | Fusing faces          | affected                   | affected                 | affected                 | unaffected                 |

