Partial 8086 disassembler for assignments 2 and 3.

Run `make test` to run the tests. The jump and loop instructions can't be tested automatically, so
run
```
make listing_0041_add_sub_cmp_jnz && cargo run < listing_0041_add_sub_cmp_jnz
```
and verify the output is reasonable for those.
