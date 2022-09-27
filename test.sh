#! /bin/bash

for i in 1 2 3 4 5 6 7 8 9 10 12 16 22 24 28 32;
   do 
   echo $i
   env RAYON_NUM_THREADS=$i ./target/release/bench
done

