# Code Review Improvements - Security Fixes and Code Quality

**Type:** fix(security), refactor(ui)  
**Scope:** Security, Code Quality  

## Changes

### Security Fixes (P0-P1)
- **WASM Plugin Sandbox**: Comprehensive sandboxing with fuel limiting, deterministic execution, directory restriction
- **Notification Blocking**: Wrapped macOS notifications in spawn_blocking, added rate limiting (5/min, 2s debounce)
- **Shell Injection Fix**: Eliminated shell injection in du command, added path validation

### Code Quality (P2-P4)
- **Clippy Auto-fixes**: Resolved formatting and import warnings
- **format_size Deduplication**: Centralized in utils.rs, removed 3 duplicate implementations
- **Let-chain Fixes**: Updated Rust 2024 let-chains for compatibility

### Deferred
- handlers.rs split (1427 lines) - requires dedicated refactoring session

### Decisions
- Error handling: AppError (domain/UI) + anyhow (application layer) pattern is acceptable

## Files Changed

- `src/plugins/mod.rs` — WASM sandboxing
- `src/presentation/services/desktop_notifications.rs` — Async notifications + rate limiting
- `src/infrastructure/brew/command.rs` — Shell injection fix
- `src/presentation/utils.rs` — New centralized utilities
- `src/presentation/components/*.rs` — format_size import updates
- `src/presentation/ui/app/*.rs` — format_size import + let-chain fixes

## Tests

All 97 tests pass after changes.