# TwoMap

## Usage

A general example is provided in the Rust doc of the TwoMap structure. 
From a meshing perspective, it corresponds to the following operations:

![TWOMAP_EXAMPLE](../images/TwoMapExample.svg)

After the creation of an initial map modeling a simple triangle, we:
- (a) add & initialize new darts to the map to model a second triangle.
- (b) 2-sew the two triangles using according to a  **StretchLeft** sewing policy.
- (c) move the most upper right vertex to form a square using both triangles.
- (d) 2-unsew the inner edge, free and remove its darts to form an actual square.

Most of those steps require multiple method calls and assertions are used to highlight
the modification done to the structure along the way.