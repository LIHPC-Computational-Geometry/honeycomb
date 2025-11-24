#!/bin/sh

SCRIPT_DIR=${HCWORKDIR}/applications
TARGET_DIR=${HCWORKDIR}/target/release

mkdir out

# Remesh (sequential)
#
# - STM overhead/cost

RAYON_NUM_THREADS=1 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_seq.out

# Remesh (strong scaling)
#
# - scalability of different operations
# - STM implementation effects

# first set of runs: 1 to 72 threads (1 chip)

RAYON_NUM_THREADS=1 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_72.out

# second set of runs: 1 to 288 threads (1 node)

RAYON_NUM_THREADS=1 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_1.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_36.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_72.out
RAYON_NUM_THREADS=108 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_108.out
RAYON_NUM_THREADS=144 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_144.out
RAYON_NUM_THREADS=180 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_180.out
RAYON_NUM_THREADS=216 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_216.out
RAYON_NUM_THREADS=252 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_252.out
RAYON_NUM_THREADS=288 ${TARGET_DIR}/remesh \
    --clip right \
    --target-length 0.01 \
    ${SCRIPT_DIR}/shape.vtk \
    1.0 1.0 \
    --n-rounds 10 \
    >> out/remesh_288.out

