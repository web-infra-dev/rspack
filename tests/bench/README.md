# Rspack JavaScript API benchmarks

The primary objective of this project is to track the performance of the Rspack JavaScript API and prevent performance degradation.

## Notes for writing benchmark cases

CPU instrument is better suited for micro-benchmarks (taking less than a second) focused on CPU-bound tasks, not system calls.
Memory instrument is used to track allocation behavior and detect memory regressions, so benchmark cases should avoid unrelated allocation noise.

System calls introduce variability in execution time. This variability is influenced by several factors, including system load, network latency, and disk I/O performance. As a result, the execution time of system calls can fluctuate significantly, making them the most inconsistent part of a program's execution time.
