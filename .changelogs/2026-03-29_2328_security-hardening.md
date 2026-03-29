# Security Hardening - Plugin Sandbox, Notification Blocking, Shell Injection Fix

**Type:** fix(security)  
**Scope:** Security  

## Changes

- **WASM Plugin Sandbox**: Implemented comprehensive sandboxing for WASM plugins with:
  - Fuel-based instruction limiting (100M max instructions)
  - Deterministic execution configuration
  - Allowed directory restriction with path validation
  - No filesystem, network, or host function access

- **Notification Blocking Fix**: Wrapped macOS notification calls in `spawn_blocking` to prevent tokio executor blocking:
  - Added rate limiting (max 5 notifications per minute)
  - Added debounce (2-second minimum interval between notifications)
  - New async API: `send_notification_async()`, `notify_*_async()`

- **Shell Injection Fix**: Eliminated shell injection vulnerability in `get_installed_sizes()`:
  - Replaced `sh -c "du -sk ..."` with direct `du` command invocation
  - Added `validate_brew_prefix()` to check for dangerous characters in paths
  - Uses `std::fs::read_dir` + per-package direct command execution

## Files Changed

- `src/plugins/mod.rs` — WASM sandboxing implementation
- `src/presentation/services/desktop_notifications.rs` — Async notifications + rate limiting
- `src/infrastructure/brew/command.rs` — Shell injection fix with path validation

## Tests

All 97 tests pass after changes.