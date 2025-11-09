## Coding guidelines

- Use format!("{var}") over format!("{}", var)
- Only use #[allow(dead_code)] when truly needed
- Favour `just` commands over `cargo`
- Guard against numeric over/underflow (use saturating ops)

## When finishing a task

- `just clippy` – fix errors
- `just fmt`
- Update docs as needed
- Persist new knowledge to memory (if available)
- Suggest improvements to AGENTS.md
- Propose next steps
