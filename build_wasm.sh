#!/bin/bash

#export EMMAKEN_CFLAGS="--js-library external.js" 
#-s \"BINARYEN_METHOD='native-wasm,asmjs'\"
#rustc --target=wasm32-unknown-emscripten brainfuck.rs -O -o brainfuck.html 
cargo +nightly test
cargo +nightly build --target=wasm32-unknown-emscripten --release
rm docs/mlisp-*.wasm docs/mlisp-*.js
cp target/wasm32-unknown-emscripten/release/mlisp.js docs
cp target/wasm32-unknown-emscripten/release/deps/* docs