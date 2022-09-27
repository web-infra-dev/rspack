#! /bin/bash

for i in 1 2 4 8 12 16 24 28 32;
   do 
   echo $i
   env RAYON_NUM_THREADS=8 WORKER_THREAD=$i hyperfine --warmup 3 ./target/release/bench
done