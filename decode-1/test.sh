#!/bin/sh

cargo build || exit 1

for FILE in listing_0037_single_register_mov.asm listing_0038_many_register_mov.asm; do
    nasm -o /dev/stdout "$FILE" | target/debug/decode-1 | diff - <(rg '^[^;]+' -or '$0' "$FILE") || {
        echo files differ, file: "$FILE"
        exit 1
    }
    echo PASSED "$FILE"
done
