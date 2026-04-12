.PHONY: test
test: listing_0039_reassembled listing_0039_more_movs listing_0040_reassembled listing_0040_challenge_movs listing_0041_reduced_reassembled listing_0041_reduced
	diff listing_0039_reassembled listing_0039_more_movs
	diff listing_0040_reassembled listing_0040_challenge_movs
	diff listing_0041_reduced_reassembled listing_0041_reduced

listing_0039_disassembled: listing_0039_more_movs build
	target/debug/sim8086 decode < $< > $@

listing_0040_disassembled: listing_0040_challenge_movs build
	target/debug/sim8086 decode < $< > $@

listing_0041_disassembled: listing_0041_add_sub_cmp_jnz build
	target/debug/sim8086 decode < $< > $@

listing_0041_reduced_disassembled: listing_0041_reduced build
	target/debug/sim8086 decode < $< > $@

listing_0039_reassembled: listing_0039_disassembled
	nasm $< -o $@

listing_0040_reassembled: listing_0040_disassembled
	nasm $< -o $@

listing_0041_reassembled: listing_0041_disassembled
	nasm $< -o $@

listing_0041_reduced_reassembled: listing_0041_reduced_disassembled
	nasm $< -o $@

listing_0041_reduced.asm: listing_0041_add_sub_cmp_jnz.asm
	# can only test these automaticlly because the jump instructions require labels
	rg -i '^(bits|add|sub|cmp)\b' $< > $@

listing_0039_more_movs.asm \
listing_0040_challenge_movs.asm \
listing_0041_add_sub_cmp_jnz.asm \
listing_0043_immediate_movs.asm \
listing_0044_register_movs.asm \
listing_0045_challenge_register_movs.asm \
listing_0046_add_sub_cmp.asm \
listing_0048_ip_register.asm \
:
	curl -O https://raw.githubusercontent.com/cmuratori/computer_enhance/refs/heads/main/perfaware/part1/$@

listing_0039_more_movs: listing_0039_more_movs.asm
	nasm $<

listing_0040_challenge_movs: listing_0040_challenge_movs.asm
	nasm $<

listing_0041_add_sub_cmp_jnz: listing_0041_add_sub_cmp_jnz.asm
	nasm $<

listing_0043_immediate_movs: listing_0043_immediate_movs.asm
	nasm $<

listing_0044_register_movs: listing_0044_register_movs.asm
	nasm $<

listing_0045_challenge_register_movs: listing_0045_challenge_register_movs.asm
	nasm $<

listing_0046_add_sub_cmp: listing_0046_add_sub_cmp.asm
	nasm $<

listing_0048_ip_register: listing_0048_ip_register.asm
	nasm $<

listing_0041_reduced: listing_0041_reduced.asm
	nasm $<

.PHONY: build
build:
	cargo build
