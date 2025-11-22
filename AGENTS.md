## Coding guidelines

- Keep `main.rs` files minimal
- Format using `cargo +nightly fmt`
- Use `format!("{var}")` over `format!("{}", var)`
- Only use `#[allow(dead_code)]` when truly needed
- Favour `just` commands over `cargo`
- Guard against numeric over/underflow (use saturating ops)

## Dependencies

- Use `cargo add` when adding new dependencies, to ensure we're using the latest compatible version
- Prefer using features that will be easier to build (e.g. rustls over openssl)
- Run `just dep-check` when changing dependencies and fix any issues

## When finishing a task

- Run `just test`
- Run `just clippy` - fix issues
- Finally, run `just fmt`
- Update docs as needed
- Add to "Learnings" section of AGENTS.md as appropriate, in this format: `- (model) YYYY-MM-DD - <learning>`
- Propose next steps

## Learnings
