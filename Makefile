AES256_DEBUG_BIN		:=target/debug/aes-256-cbc
AES256_RELEASE_BIN		:=target/release/aes-256-cbc
AES256_BIN			:=$(AES256_DEBUG_BIN)
BIP39_DEBUG_BIN			:=target/debug/bip39
BIP39_RELEASE_BIN		:=target/release/bip39
BIP39_BIN			:=$(BIP39_DEBUG_BIN)
SLUGIFY_FILENAMES_DEBUG_BIN	:=target/debug/slugify-filenames
SLUGIFY_FILENAMES_RELEASE_BIN	:=target/release/slugify-filenames
SLUGIFY_FILENAMES_BIN		:=$(SLUGIFY_FILENAMES_DEBUG_BIN)
PASSWORD			:="I X@X@ Nickelback <3"
PLAINTEXT			:="Hello World"

all: fmt release

clean:
	rm -f *.aes *.yaml

cls:
	@echo -e "\033[H\033[2J"

release:
	cargo build --release
	cp target/release/slugify-filenames ~/usr/bin/
	cp target/release/aes-256-cbc ~/usr/bin/
	cp target/release/bip39 ~/usr/bin/

fmt:
	rustfmt --edition 2021 src/*.rs
tmp:
	@rm -rf tmp
	@mkdir -p tmp/{Foo,BAR,BaZ,}/{One,TWO,THree@FouR}
	@for name in $$(find tmp -type d); do uuidgen > $$name/AA; done
	@for name in $$(find tmp -type d); do uuidgen > $$name/bB; done
	@for name in $$(find tmp -type d); do uuidgen > $$name/Cc; done
	@for name in $$(find tmp -type f); do uuidgen > $$name; done

dry-run:tmp
	cargo run --bin slugify-filenames -- -r tmp --dry-run

test: test-aes-256 test-slugify-filenames

test-slugify-filenames: tmp cls
	cargo run --bin slugify-filenames -- -r tmp --dry-run
	cargo run --bin slugify-filenames -- -r tmp

test-aes-256: aes-256-key aes-256-password


build:
	cargo build

silent: tmp cls
	cargo run --bin slugify-filenames -- -r tmp --silent


coverage: cls
	grcov . --binary-path target/debug/slugify-filenames -s . -t html --branch --ignore-not-existing -o ./coverage/

aes-256-ask: cls build
	@echo $$(seq 10 | sed 's/[0-9]*/-/g' | tr '\n' '-')
	@echo "$@"
	@echo $$(seq 10 | sed 's/[0-9]*/-/g' | tr '\n' '-')
	@echo $(PASSWORD) | pbcopy
	@echo "\033[38;5;227mPASSWORD COPIED TO CLIPBOARD: \033[38;5;49m"$(PASSWORD)"\033[0m"
	$(AES256_BIN) encrypt --ask-password --output-filename Cargo.toml.aes --input-filename Cargo.toml
	$(AES256_BIN) decrypt --ask-password --input-filename Cargo.toml.aes --output-filename Cargo.toml
	cargo check

aes-256-key: cls build
	@echo $$(seq 10 | sed 's/[0-9]*/-/g' | tr '\n' '-')
	@echo "$@"
	@echo $$(seq 10 | sed 's/[0-9]*/-/g' | tr '\n' '-')
	$(AES256_BIN) generate --key-filename ~/aes-256-cbc.yaml --password $(PASSWORD)
	$(AES256_BIN) encrypt --key-filename ~/aes-256-cbc.yaml --output-filename Cargo.toml.aes --input-filename Cargo.toml
	$(AES256_BIN) decrypt --key-filename ~/aes-256-cbc.yaml --input-filename Cargo.toml.aes --output-filename Cargo.toml
	cargo check

aes-256-password: cls build
	@echo $$(seq 10 | sed 's/[0-9]*/-/g' | tr '\n' '-')
	@echo "$@"
	@echo $$(seq 10 | sed 's/[0-9]*/-/g' | tr '\n' '-')
	$(AES256_BIN) encrypt --password $(PASSWORD) --output-filename Cargo.toml.aes --input-filename Cargo.toml
	$(AES256_BIN) decrypt --password $(PASSWORD) --input-filename Cargo.toml.aes --output-filename Cargo.toml
	cargo check

aes-256: aes-256-key aes-256-password aes-256-ask

load: clean build
	./aestest.sh


$(AES256_RELEASE_BIN):
	cargo build --release

$(AES256_DEBUG_BIN):
	cargo build



.PHONY: all release fmt tmp test dry-run coverage aes256 build clean test-e2e test-aes-256 test-slugify-filenames
