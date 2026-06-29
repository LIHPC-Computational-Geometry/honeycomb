#!/bin/sh

SCRIPT_DIR=${HCWORKDIR}/applications
TARGET_DIR=${HCWORKDIR}/target/release

mkdir out

# Delaunay (sequential)
#
# - STM overhead/cost
# - algorithm perf

${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    98304 \
    >> out/delaunay_seq.out
${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    512 \
    >> out/delaunay_seq_1.out
${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    4608 \
    >> out/delaunay_seq_9.out
${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    9216 \
    >> out/delaunay_seq_18.out
${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    18432 \
    >> out/delaunay_seq_36.out
${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    27648 \
    >> out/delaunay_seq_54.out
${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    36864 \
    >> out/delaunay_seq_72.out


# Delaunay (weak scaling, large starting triangulation)
#
# 8192 points per thread, work divided into rounds via BRIO
#
# - scalability
# - algorithm perf

# 1 -> 72

RAYON_NUM_THREADS=1 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    32768 \
    >> out/delaunay_ws_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    98304 \
    >> out/delaunay_ws_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    294912 \
    >> out/delaunay_ws_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    589824 \
    >> out/delaunay_ws_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    1179648 \
    >> out/delaunay_ws_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    1769472 \
    >> out/delaunay_ws_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    2359296 \
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
    1 1 1 \
    589824 \
    >> out/delaunay_ss_1.out
RAYON_NUM_THREADS=3 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    589824 \
    >> out/delaunay_ss_3.out
RAYON_NUM_THREADS=9 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    589824 \
    >> out/delaunay_ss_9.out
RAYON_NUM_THREADS=18 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    589824 \
    >> out/delaunay_ss_18.out
RAYON_NUM_THREADS=36 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    589824 \
    >> out/delaunay_ss_36.out
RAYON_NUM_THREADS=54 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    589824 \
    >> out/delaunay_ss_54.out
RAYON_NUM_THREADS=72 ${TARGET_DIR}/incremental-delaunay \
    1 1 1 \
    589824 \
    >> out/delaunay_ss_72.out
