These are fuzz tests that can be run with cargo-fuzz by typing `cargo fuzz run <fuzz_test>` (where fuzz_test is one of the files in the `fuzz_targets` folder).

I (the maintainer of this library) try to run all of these for at least 10-20 million iterations before publishing a new release.
This helps find bugs that could lead to crashes or other bugs like infinite loops.
