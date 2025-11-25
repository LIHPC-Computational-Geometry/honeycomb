#!/bin/sh

SCRIPT_DIR=${HCWORKDIR}/applications
TARGET_DIR=${HCWORKDIR}/target/release

mkdir out

# Delaunay (sequential)
#
# - STM overhead/cost
# - algorithm perf

${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    32 \
    --init-points 1024 \
    >> out/delaunay_seq.out

# Delaunay (weak scaling, minimal triangulation)
#
# - conflict cost
# - algorithm perf

# 1 -> 72

RAYON_NUM_THREADS=1 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    2048 \
    >> out/delaunay_min_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    6144 \
    >> out/delaunay_min_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    18432 \
    >> out/delaunay_min_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    36864 \
    >> out/delaunay_min_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    73728 \
    >> out/delaunay_min_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    110592 \
    >> out/delaunay_min_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    >> out/delaunay_min_72.out

# Delaunay (weak scaling, large triangulation)
#
# - scalability
# - algorithm perf

# 1 -> 72

RAYON_NUM_THREADS=1 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    2048 \
    --init-points 512 \
    >> out/delaunay_sc_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    6144 \
    --init-points 1536 \
    >> out/delaunay_sc_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    18432 \
    --init-points 4608 \
    >> out/delaunay_sc_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    36864 \
    --init-points 9216 \
    >> out/delaunay_sc_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    73728 \
    --init-points 18432 \
    >> out/delaunay_sc_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    110592 \
    --init-points 27648 \
    >> out/delaunay_sc_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    --init-points 36864 \
    >> out/delaunay_sc_72.out

# 1 -> 288

# TODO: adjust problem size, and use tet grid init
#
#
# TODO: complete
