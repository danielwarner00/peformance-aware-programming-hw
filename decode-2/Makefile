.PHONY: test
test: listing_0039_reassembled listing_0040_reassembled
	diff listing_0039_reassembled listing_0039_more_movs
	diff listing_0040_reassembled listing_0040_challenge_movs

listing_0039_disassembled: listing_0039_more_movs build
	target/debug/decode-2 < $< > $@

listing_0040_disassembled: listing_0040_challenge_movs build
	target/debug/decode-2 < $< > $@

listing_0039_reassembled: listing_0039_disassembled
	nasm $< -o $@

listing_0040_reassembled: listing_0040_disassembled
	nasm $< -o $@

listing_0039_more_movs.asm listing_0040_challenge_movs.asm:
	curl -O https://raw.githubusercontent.com/cmuratori/computer_enhance/refs/heads/main/perfaware/part1/$@

listing_0039_more_movs: listing_0039_more_movs.asm
	nasm $<

listing_0040_challenge_movs: listing_0040_challenge_movs.asm
	nasm $<

.PHONY: build
build:
	cargo build
