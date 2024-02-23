# Orbits

We define orbits as a set of darts that are accessible from a given dart, 
using a certain set of beta functions. For example:

- *⟨β<sub>1</sub>⟩(d)* refers to all darts accessible from *d* using 
  *β<sub>1</sub>* recursively any number of times.
- *⟨β<sub>1</sub>, β<sub>3</sub>⟩(d)* refers to all darts accessible 
  from *d* using any combination of *β<sub>1</sub>* and *β<sub>3</sub>*.

## *i*-cells

A specific subset of orbits, referred to as *i*-cells are defined and often 
used in algorithms. The general definition is the following: 

- **if i = 0**:  *0-cell(d) = ⟨{ β<sub>j</sub> o β<sub>k</sub> with 1 ≤ j < k ≤ N }⟩(d)*
- **else**: *i-cell(d) = ⟨β<sub>1</sub>, β<sub>2</sub>, ..., β<sub>i-1</sub>, β<sub>i+1</sub>, ..., β<sub>N</sub>⟩(d)*

In our case, we can use specialized definitions for our dimensions:

| *i* | Geometry | 2-map                                                                                       | 3-map                                                                                                                                                     |
|-----|----------|---------------------------------------------------------------------------------------------|-----------------------------------------------------------------------------------------------------------------------------------------------------------|
| 0   | Vertex   | *⟨β<sub>1</sub> o β<sub>2</sub>⟩(d)* <br> **or** <br> *⟨β<sub>2</sub> o β<sub>-1</sub>⟩(d)* | *⟨β<sub>3</sub> o β<sub>2</sub>, β<sub>1</sub> o β<sub>3</sub>⟩(d)* <br> **or** <br> *⟨β<sub>3</sub> o β<sub>2</sub>, β<sub>3</sub> o β<sub>-1</sub>⟩(d)* |
| 1   | Edge     | *⟨β<sub>2</sub>⟩(d)*                                                                        | *⟨β<sub>2</sub>, β<sub>3</sub>⟩(d)*                                                                                                                       |
| 2   | Face     | *⟨β<sub>1</sub>⟩(d)*                                                                        | *⟨β<sub>1</sub>, β<sub>3</sub>⟩(d)*                                                                                                                       |
| 3   | Volume   | -                                                                                           | *⟨β<sub>1</sub>, β<sub>2</sub>⟩(d)*                                                                                                                       |