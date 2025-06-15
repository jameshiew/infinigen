On Startup
1 . List available MCP tools
2 . Read Cargo.toml; note newest crate versions you are familiar with
3 . Outline the task and how much you’ll tackle before QA
4 . Use external look-ups early (fetch or other tooling) to verify APIs

Logging & Progress
• Comment liberally, leave TODOs
• Regularly output estimated progress, e.g. // ~10 % done (may rise or fall)

Rust Coding Rules
• Keep modules small; avoid dumping code in one file
• Use format!("{var}") over format!("{}", var)
• Only use #[allow(dead_code)] when truly needed
• Compile/test often with `just check`; favour `just` over `cargo`
• Guard against numeric over/underflow (use saturating ops)

Finishing Checklist
1 . `just clippy` – fix errors
2 . `just fmt`
3 . Update docs as needed
4 . Persist new knowledge to memory
5 . Suggest improvements to AGENTS.md
6 . Propose next steps

General Tips
• Work incrementally; compile early & often.
