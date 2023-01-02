# RIIR dlt-daemon

This is the repository for a pure Rust implementation of Diagnostic Log and Trace. The original 
implementation is in C and C++, and is located at https://github.com/COVESA/dlt-daemon .

## Milestones

1. Implement Rust library to log messages to COVESA DLT-Daemon. 
2. Implement C API to wrap around the Rust implementation. The C-Api shall be a drop-in replacement for COVESA DLT user library.
3. Implement DLT-Daemon 
4. Implement DLT-System


## Requirements
1. Configuration files shall follow identical syntax
2. Command line switches must be identical
3. COVESA DLT compile time swithes to be converted to Rust features.






