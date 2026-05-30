When compiling zerostack:
- Never run `cargo build` or `cargo build --release`
- Always run `cargo fmt`
- Always run `cargo install --path .`
- Run `cargo test` if you want to check all unit tests

Always write tests when writing new non-TUI code.
Always update docs/ files when needed.
If adding or editing slash commands, edit the slash commands `/` picker in the TUI.
