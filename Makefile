.PHONY: css css-watch desktop web check test clean

# Build Tailwind CSS (run after changing input.css or adding new classes)
css:
	npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css

# Watch mode — auto-rebuild CSS on changes
css-watch:
	npx @tailwindcss/cli -i ./input.css -o ./assets/tailwind.css --watch

# Serve desktop app (run css first if styles changed)
desktop: css
	dx serve --platform desktop

# Serve web app (run css first if styles changed)
web: css
	dx serve --platform web

# Check compilation for all targets
check:
	cargo check
	cargo check --target wasm32-unknown-unknown -p zk-vault-ui

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean
	rm -f assets/tailwind.css
