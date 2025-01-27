.PHONY: all clean build install install-tools install-deps check-deps

all: build 

clean:
	rm -rf node_modules target yarn.lock package-lock.json Cargo.lock

build: check-deps
	cargo tauri build

dev: check-deps
	cargo tauri dev

install: install-deps install-tools

install-deps:
	@echo "Installing JS dependencies..."
	@if test -f yarn.lock; then \
		yarn install; \
	else \
		npm install; \
	fi

install-tools:
	@echo "Installing Trunk and Tauri-CLI"
	cargo install trunk tauri-cli@1.6.5
	@echo "Adding WebAssembly target"
	rustup target add wasm32-unknown-unknown

check-deps:
	@echo "Checking NPM dependencies..."
	@test -d node_modules
	@echo "Node Modules found."
	@echo "Checking Trunk Version..."
	@trunk -V
	@echo "Trunk found."
	@echo "Checking Tauri-CLI Version..."
	@vnum=$$(cargo tauri -V | awk '{print $$2}') ; \
	if ! [ "$$vnum" = "1.6.5" ] ; then \
		echo "Wrong Tauri-CLI version found."; \
		false; \
	fi
	@echo "Right Tauri-CLI version found."
