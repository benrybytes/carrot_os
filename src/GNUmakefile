# Nuke built-in rules and variables.
MAKEFLAGS += -rR
.SUFFIXES:

# This is the name that our final executable will have.
# Change as needed.
override OUTPUT := carrot_os

# Convenience macro to reliably declare user overridable variables.
override USER_VARIABLE = $(if $(filter $(origin $(1)),default undefined),$(eval override $(1) := $(2)))

# Target architecture to build for. Default to x86_64.
$(call USER_VARIABLE,KARCH,x86_64)

ifeq ($(RUST_TARGET),)
    # override RUST_TARGET := $(KARCH)-unknown-none
    override RUST_TARGET := x86_64_carrot_os

	ifeq ($(KARCH),riscv64)
    	override RUST_TARGET := riscv64gc-unknown-none-elf
	endif
endif

ifeq ($(RUST_PROFILE),)
    override RUST_PROFILE := dev
endif

override RUST_PROFILE_SUBDIR := $(RUST_PROFILE)
ifeq ($(RUST_PROFILE),dev)
    override RUST_PROFILE_SUBDIR := debug
endif

# Default target.
.PHONY: all
all:
	cargo build --target $(RUST_TARGET).json --profile $(RUST_PROFILE)
	cp ../target/$(RUST_TARGET)/debug/carrot_os ../carrot_os

# Remove object files and the final executable.
.PHONY: clean
clean:
	cargo clean
	rm -rf carrot_os

.PHONY: distclean
distclean: clean
