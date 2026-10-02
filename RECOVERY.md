# Recovered custom Tock

Upstream: https://github.com/akirco/tock
Base commit: be140753b9d2a0c353524a51a914f2ef2f960713

The src/ui.rs patch was recovered from the final successful Git diff in the owner's July 9, 2026 local development history. It fixes digit cell widths, digit spacing, timer centering, and independent subtitle centering. No private chat history or settings are included here.

Build: `cargo build --release --locked`
Install: `cargo install --path . --locked`

The historical installed macOS arm64 binary has SHA-256 55504536e65daadd02395930f5e276f6be97e9652453e9097bb9e057146c433f. Rebuilding with a newer toolchain need not produce the same bytes.
