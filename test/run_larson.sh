#!/bin/bash
if [[ $# -ne 1 ]]; then
  ALLOC="r"
else
  ALLOC=$1
fi
ARGS="ALLOC="
ARGS=${ARGS}${ALLOC}
echo $ARGS

# make clean
# make larson_test ${ARGS} FLAGS='-DSHM_SIMULATING'
# if [ $ALLOC="r" ]; then
# 	mv larson_test ./ralloc_bench
# fi
rm -rf larson.csv
echo "thread,ops,allocator" >> larson.csv
for i in {1..3}; do
	for threads in 1 2 4 6 10 16 20 24 32 40 48 62 72 80 84 88; do
		rm -rf /mnt/pmem/*
		rm -rf /mnt/cxl_mem/*
		rm -rf /dev/shm/*
		if [ $ALLOC="r" ]; then
			BINARY=./ralloc_bench/larson_test ./larson-single.sh $threads $ALLOC
		fi
	done
done
# SEDARGS="2,\$s/$/"
# SEDARGS=${SEDARGS}","${ALLOC}"/"
# echo $SEDARGS
# sed ${SEDARGS} -i larson.csv
NAME="../data/larson/larson_"${ALLOC}".csv"
cp larson.csv ${NAME}
