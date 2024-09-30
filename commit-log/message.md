# Title

Profiled Program for Hot Spots & Implemented Optimizations

# Summary

The program was profiled using `perf` and hotspots were located using the resulting flamegraph. After optimizations were implemented for each identified hotspot, a total speedup of 14.3x was achieved on `ecetesla3`. Optimizations include reducing file I/O for idea & package threads, splitting idea & package communication channel into two seperate channels, and reducing lock contention on checksum updates by giving each thread their own local checksum.

# Technical details

### Optimization 1: File I/O
Every PackageDownloader was reading `packages.txt` for every package. There is a lot of overhead in loading a file from the disk so to reduce this overhead, I read the file a single time outside of the threads and then passed it in as a parameter.

Similarily, IdeaGenerators were reading `ideas-products.txt` & `ideas-consumers.txt` for every idea. Not only that, but were recomputing the crossproduct of the two files for every idea, as well. I changed it so files are read a single time outside of the threads and the result of the crossproduct is passed in as a parameter.

### Optimization 2: Thread Communication
Students when recieving events must all block on the same channel, whether or not they currently need a new idea. This causes a problem where a student already has an idea and needs packages but is only getting new ideas from the channel. Having to perform a `recv()` and a `send()` on the channel for an event that is not needed is a waste.

The solution was to split up the channel into an `IdeaEvent` channel and a `PkgEvent` channel. This allows students to only access the `IdeaEvent` channel when they don't have an idea already. Changing channel access to non-blocking seemed to also perform better than simply blocking on the channel, as well.

### Optimization 3: Checksum Updates
Threads were updating shared checksums very frequently causing a lot of time to be wasted in lock contention. The solution to this was to give each thread their own local checksum to update and then return the result. Checksums for each thread would then be combined in the main thread. This completely avoids the need for synchronization.

Printing the intermediate checksum values were also removed since `stdout()` was having lock contention issues and the information was not needed.

# Testing for correctness
Correctness was tested by running on ECE servers and making sure the global checksums remained the same as the unoptimized version. This is sufficient since the unoptimized version is known to be correct. The program was also checked for memory leaks with valgrind memcheck and no errors were found.

# Testing for performance.
All testing was done on `ecetesla3` with default parameters. I began with a baseline benchmark using hyperfine resulting in an average of 1.684s over many runs. I then ran `make` to generate a flamegraph & ran `perf record` to get a finer look at what specific functions/lines are using the most resources. Based on the flamegraph `PackageDownloader::run` was using the most resources, more specifically the `std::fs::read_to_string` & `core::iter::traits::iterator::Iterator::nth` child functions. Implementing Optimization 1 fixed this problem and is confirmed by generating a new flamegraph showing that the number of samples in `PackageDownloader::run` has been reduced. Hyperfine also shows a speedup from baseline.

Next I wanted to change how the threads were communicating since `crossbeam_channel::channel::Receiver<T>::recv` & `crossbeam_channel::channel::Sender<T>::send` were using a lot of resources throughout the whole program. Implementing Optimization 2 fixed this issue and is confirmed the same way as Optimization 1. Hyperfine shows another speedup.

Since everything is moving much faster now, I noticed that a lot of resources were being used by `std::sync::mutex::Mutex<T>::lock` when threads wanted to update the shared checksum. Implementing Optimization 3 fixed this and hyperfine showed a final runtime of 117.4ms.

Baseline:
```console
ecetesla3:~/ece459-1241-a4-brayner> hyperfine -i "target/release/lab4"
Benchmark #1: target/release/lab4
  Time (mean ± σ):      1.684 s ±  0.384 s    [User: 926.2 ms, System: 97.7 ms]
  Range (min … max):    1.138 s …  2.142 s    10 runs
```

Optimized:
```console
ecetesla3:~/ece459-1241-a4-brayner> hyperfine -i "target/release/lab4" 
Benchmark #1: target/release/lab4
  Time (mean ± σ):     117.4 ms ±  29.6 ms    [User: 69.7 ms, System: 1.7 ms]
  Range (min … max):    68.6 ms … 201.9 ms    31 runs
```
