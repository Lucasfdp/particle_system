# particle_system — one-line setup: `make` checks/installs dependencies, then builds (§ 17).
#
#   make              deps + release build
#   make run          run with N particles (default 1000000), e.g. make run N=3m
#   make deps         check for (and where possible install) Rust, glslc, Vulkan
#   make sdk          download the LunarG Vulkan SDK (glslc) into your home folder — no admin
#   make spv          refresh the committed SPIR-V fallback in engine/shaders/spv/
#   make test         unit tests
#   make clean / fclean / re

N ?= 1000000
SDK_DIR := $(HOME)/.local/vulkan-sdk

# Tools installed into the home folder (rustup, the SDK) must be found even in the
# same `make` run that installed them.
SDK_BIN := $(firstword $(wildcard $(SDK_DIR)/*/x86_64/bin))
export PATH := $(HOME)/.cargo/bin:$(if $(SDK_BIN),$(SDK_BIN):)$(PATH)

SHADERS := $(wildcard engine/shaders/*.vert engine/shaders/*.frag engine/shaders/*.comp)

all: deps build

deps:
	@sh scripts/deps.sh

build:
	cargo build --release

run: build
	./target/release/particle_system $(N)

test:
	cargo test

sdk:
	@sh scripts/deps.sh --install-sdk

spv:
	@command -v glslc >/dev/null || { echo "glslc not found: run 'make deps' (or 'make sdk')"; exit 1; }
	@mkdir -p engine/shaders/spv
	@for s in $(SHADERS); do \
		glslc "$$s" -o "engine/shaders/spv/$$(basename $$s).spv" && echo "compiled $$s"; \
	done

clean:
	cargo clean -p particle_system -p engine

fclean:
	cargo clean

re: fclean all

.PHONY: all deps build run test sdk spv clean fclean re
