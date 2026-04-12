.PHONY: test
test: \
build/disassembled_listing_0039_more_movs.bin build/listing_0039_more_movs.bin \
build/disassembled_listing_0040_challenge_movs.bin build/listing_0040_challenge_movs.bin \
build/disassembled__listing_0041_reduced.bin build/_listing_0041_reduced.bin
	diff build/disassembled_listing_0039_more_movs.bin build/listing_0039_more_movs.bin
	diff build/disassembled_listing_0040_challenge_movs.bin build/listing_0040_challenge_movs.bin
	diff build/disassembled__listing_0041_reduced.bin build/_listing_0041_reduced.bin

build/disassembled_%.asm: build/%.bin target/debug/sim8086 | build
	target/debug/sim8086 decode < $< > $@

build/_listing_0041_reduced.asm: build/listing_0041_add_sub_cmp_jnz.asm | build
	# can only test these automaticlly because the jump instructions require labels
	rg -i '^(bits|add|sub|cmp)\b' $< > $@

build:
	mkdir build

build/listing_%.asm: | build
	curl https://raw.githubusercontent.com/cmuratori/computer_enhance/refs/heads/main/perfaware/part1/$(notdir $@) -o $@

build/%.bin: build/%.asm | build
	nasm $< -o $@

.PHONY: target/debug/sim8086
target/debug/sim8086:
	cargo build

.PHONY: clean
clean:
	rm -rf build
	cargo clean

.PHONY: execute
execute: build/$(BIN) target/debug/sim8086
	target/debug/sim8086 execute < build/$(BIN)

.PHONY: build/image.data
build/image.data: target/debug/sim8086 build/listing_0054_draw_rectangle.bin | build
	target/debug/sim8086 execute --dump $@ < build/listing_0054_draw_rectangle.bin
