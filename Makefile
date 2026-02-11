.PHONY: run build install clean

# Default: compile and run the desktop app
run:
	npm run tauri dev

# Build production binaries (creates platform-specific installer)
build:
	npm run tauri build

# Install dependencies (run once before first use)
install:
	npm install

# Clean build artifacts
clean:
	rm -rf target
	rm -rf dist
	rm -rf src-tauri/target
