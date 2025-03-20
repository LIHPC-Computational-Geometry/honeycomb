# Remeshing routines

---

Remeshing pipeline are comprised of multiple phases, each having their own operation type. We group the implementation
of such routines in [this](../../honeycomb_kernels/remeshing/index.html) module.

TODO: make a figure of the pipeline

## Vertex relaxation

- `move_vertex_to_average` -- move a vertex to the average osition of a passed list.
- `dvr` -- not yet implemented (https://github.com/LIHPC-Computational-Geometry/honeycomb/pull/212)

## Cell division

- `cut_inner_edge` / `cut_outer_edge` -- cut an edge in half and build triangles from the new vertex

## Cell fusion

- `collapse_edge` -- not yet implemented

## Cell edition

- `swap_edge` -- tip over an edge shared by two triangles

## Quality

- `compute_face_skewness_2d` / `compute_face_skewness_3d` -- compute the [skewness][SKW] of a given face.

[SKW]: https://ansyshelp.ansys.com/public/account/secured?returnurl=//////Views/Secured/corp/v242/en/wb_msh/msh_skewness.html
