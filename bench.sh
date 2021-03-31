#!/bin/bash

time ./target/release/main -iSpectralPath -o "" ./prism.ron

# run 1
### fma: off
## RUNTIME: 0:58.175

# run 2
### fma: on
## RUNTIME: 0:56.597s