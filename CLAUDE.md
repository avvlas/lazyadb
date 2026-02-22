# CLAUDE.md — Project Instructions

LazyADB is a TUI for Android Debug Bridge built in Rust with ratatui.

## Architecture:

- This app uses Component architecture (see Component trait in `src/components/mod.rs`)
- There are 2 types of components right now: Panes (always visible in the ui) and Modals (displayed as overlays, can be dismissed)
- Crossterm events are handled in `src/tui.rs` and passed to the App as `Event` enum. 
- App handles events in a loop and sends `Msg` messages to components in `update` method.
- Components change their state in `update` and return vector of `Command` to the app. 

### State ownership

- App (`app.rs`) is a thin dispatcher — it should NOT store component state (e.g. selected device).
- Each component owns and manages its own state. When a state change needs to be communicated, the component returns a `Command` to the app.
- App reacts to commands by performing side-effects (ADB calls, sending `Msg` to other components) — never by duplicating state that a component already tracks.

## Code conventions

- Use `color-eyre` for error handling (`Result` type)
- Use `tracing` for logging.
- Keep functions short and clean (SRP). 
- Follow Rust best-practices and conventions. 
- Use meaningful variable and function names.
- Write clear and concise comments **only** when really necessary (i.e. code would not be readable without comments)
- Use rustfmt to automate import formatting

### Types design

- Keep types focused on a single responsibility
- Derive common traits: Debug, Clone, PartialEq where appropriate
- Use #[derive(Default)] when a sensible default exists
- Prefer composition over inheritance-like patterns
- Use builder pattern for complex struct construction
- Make fields private by default; provide accessor methods when needed

### Memory and Performance

- Avoid unnecessary allocations; prefer &str over String when possible
- Prefer stack allocation over heap when appropriate
- Use Arc and Rc judiciously; prefer borrowing

### Tools

- Use rustfmt to format code.
- Use clippy to lint and follow it's suggestions.

### Testing

- Write unit tests for all new functions and types
- `cargo build` must always succeed
- Use #[cfg(test)] modules for test code

## Commit style

- Conventional messages: `feat:`, `fix:`, `refactor:`, `docs:`
- One logical change per commit
