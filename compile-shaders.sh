#!/bin/sh

SHADERS_DIR="shaders"
BUILD_DIR="compiled-shaders"

mkdir -p $BUILD_DIR

GREEN="\033[38;5;2m"
RESET="\033[0m"

for file in $SHADERS_DIR/*
do
    filename=$(basename $file)
    if [ $filename = "prelude.comp" ]
    then
        continue
    fi
    basename=${filename%.*}
    format=${filename#*.}
    out="$BUILD_DIR/$basename-$format.spv"
    echo "== Compiling $GREEN${file}$RESET to $GREEN${out}$RESET =="
    glslc $file -o $out
    if [ $? -ne 0 ]
    then
        echo "Encountered one error, stopping"
        exit 1
    fi
done
