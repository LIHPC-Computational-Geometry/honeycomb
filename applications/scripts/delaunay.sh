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
    2048 \
    >> out/delaunay_seq.out


# Delaunay (weak scaling, large starting triangulation)
#
# 8192 points per thread, work divided into rounds via BRIO
#
# - scalability
# - algorithm perf

# 1 -> 72

RAYON_NUM_THREADS=1 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    8192 \
    >> out/delaunay_ws_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    24576 \
    >> out/delaunay_ws_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    73728 \
    >> out/delaunay_ws_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    >> out/delaunay_ws_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    294912 \
    >> out/delaunay_ws_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    442368 \
    >> out/delaunay_ws_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    589824 \
    >> out/delaunay_ws_72.out

# Strong scaling
# 
# work divided into rounds via BRIO
#
# - scalability
# - algorithm perf
# - conflict rate

# 1 -> 72

RAYON_NUM_THREADS=1 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    >> out/delaunay_ss_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    >> out/delaunay_ss_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    >> out/delaunay_ss_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    >> out/delaunay_ss_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    >> out/delaunay_ss_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    >> out/delaunay_ss_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/incremental-delaunay \
    10 10 10 \
    147456 \
    >> out/delaunay_ss_72.out
