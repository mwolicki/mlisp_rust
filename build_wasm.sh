#!/bin/bash

#export EMMAKEN_CFLAGS="--js-library external.js" 
#-s \"BINARYEN_METHOD='native-wasm,asmjs'\"
#rustc --target=wasm32-unknown-emscripten brainfuck.rs -O -o brainfuck.html 
cargo +nightly build --target=wasm32-unknown-emscripten --release
rm public/mlisp-*.wasm public/mlisp-*.js
cp target/wasm32-unknown-emscripten/release/mlisp.js public
cp target/wasm32-unknown-emscripten/release/deps/* public