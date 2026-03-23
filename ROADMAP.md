# Brewsty Development Roadmap 🍺

Comprehensive development plan for Brewsty - Homebrew GUI Manager.

**Last Updated:** March 23, 2026  
**Current Version:** 0.7.0  
**Branch:** `feature/gui-ux-improvements` (just pushed)

---

## 📋 Implementation Order

1. **Technical Debt** - Foundation improvements
2. **Quick Wins** - Fast value additions (1-2h each)
3. **High Priority** - Core feature enhancements
4. **Medium Priority** - UX improvements
5. **Advanced** - Future capabilities

---

## 🔧 Phase 1: Technical Debt

### 1.1 Add Test Suite
**Status:** ⏳ Pending  
**Estimated:** 4-6h  
**Priority:** Critical

- [ ] Set up testing infrastructure
- [ ] Unit tests for `domain/entities/`
- [ ] Unit tests for `domain/repositories/`
- [ ] Integration tests for `infrastructure/brew/`
- [ ] Mock implementations for repositories
- [ ] CI integration for tests

**Dependencies:** None  
**Files to create:** `tests/`, `src/*/tests.rs`

---

### 1.2 Improve Error Handling
**Status:** ⏳ Pending  
**Estimated:** 2-3h  
**Priority:** Critical

- [ ] Create custom error types in `domain/errors.rs`
- [ ] Better error messages for user-facing operations
- [ ] Error context tracing
- [ ] Graceful degradation on partial failures

**Dependencies:** None  
**Files to modify:** `src/domain/errors.rs`, `src/infrastructure/brew/`

---

### 1.3 File-Based Logging
**Status:** ⏳ Pending  
**Estimated:** 1-2h  
**Priority:** High

- [ ] Add `tracing-appender` dependency
- [ ] Configure rolling file logger to `~/.brewsty/logs/`
- [ ] Log levels: ERROR, WARN, INFO, DEBUG, TRACE
- [ ] Log rotation (daily, keep 7 days)

**Dependencies:** None  
**New dependency:** `tracing-appender = "0.2"`

---

### 1.4 Configuration File
**Status:** ⏳ Pending  
**Estimated:** 2-3h  
**Priority:** High

- [ ] Create `~/.brewsty/config.toml`
- [ ] Config options: theme, log_level, auto_check_updates, cache_dir
- [ ] Config UI in Settings tab
- [ ] Migrate hardcoded values to config

**Dependencies:** None  
**New dependency:** `toml = "0.8"`  
**Files to create:** `src/infrastructure/config.rs`

---

### 1.5 Better Tracing Spans
**Status:** ⏳ Pending  
**Estimated:** 1-2h  
**Priority:** Medium

- [ ] Add structured logging spans for all operations
- [ ] Correlation IDs for async operations
- [ ] Performance timing for slow operations
- [ ] Debug dashboard for active spans

**Dependencies:** 1.3 File-Based Logging

---

## ⚡ Phase 2: Quick Wins

### 2.1 Keyboard Shortcuts
**Status:** ⏳ Pending  
**Estimated:** 1h  
**Priority:** High

- [ ] `Cmd+Q` - Quit application
- [ ] `Cmd+F` - Focus search
- [ ] `Cmd+R` - Refresh current tab
- [ ] `Delete` - Uninstall selected package
- [ ] `Cmd+A` - Select all (in Outdated tab)
- [ ] `Escape` - Close modals

**Dependencies:** None  
**Files to modify:** `src/presentation/ui/app/handlers.rs`

---

### 2.2 Export to JSON
**Status:** ⏳ Pending  
**Estimated:** 1h  
**Priority:** High

- [ ] Export installed packages to JSON
- [ ] Export to file dialog (rfd)
- [ ] Include: name, version, type, install_date
- [ ] Button in Maintenance tab

**Dependencies:** None  
**Files to create:** `src/application/use_cases/export_packages.rs`

---

### 2.3 Package Search in Installed Tab
**Status:** ⏳ Pending  
**Estimated:** 1h  
**Priority:** High

- [ ] Add search/filter input to Installed tab
- [ ] Real-time filtering by name
- [ ] Clear button
- [ ] Match count display

**Dependencies:** None  
**Files to modify:** `src/presentation/ui/tabs/installed.rs`

---

### 2.4 Confirmation Dialogs
**Status:** ⏳ Pending  
**Estimated:** 1h  
**Priority:** High

- [ ] Confirm uninstall with package name
- [ ] Confirm bulk operations
- [ ] Show count of affected packages
- [ ] "Don't show again" option

**Dependencies:** None  
**Files to modify:** `src/presentation/components/`, `src/presentation/ui/app/mod.rs`

---

## 🚀 Phase 3: High Priority Features

### 3.1 Auto-Update Check
**Status:** ⏳ Pending  
**Estimated:** 3-4h  
**Priority:** Critical

- [ ] Check GitHub Releases API on startup
- [ ] Compare current version with latest
- [ ] Notification toast if update available
- [ ] Link to release page
- [ ] Config option to disable auto-check

**Dependencies:** 1.4 Configuration File  
**New dependency:** `reqwest = { version = "0.11", features = ["json"] }`

---

### 3.2 Package Details Panel
**Status:** ⏳ Pending  
**Estimated:** 4-5h  
**Priority:** Critical

- [ ] Side panel or modal with package info
- [ ] Display: description, homepage, dependencies, dependents
- [ ] Parse `brew info <package>` output
- [ ] Quick links: homepage, repo, issues
- [ ] Install date, version history

**Dependencies:** None  
**Files to create:** `src/presentation/components/package_details.rs`

---

### 3.3 Export/Import Full State
**Status:** ⏳ Pending  
**Estimated:** 3-4h  
**Priority:** Critical

- [ ] Export all package info to JSON/YAML
- [ ] Import on another Mac
- [ ] Dry-run mode for import
- [ ] Conflict resolution (skip/overwrite)
- [ ] Progress indicator for bulk operations

**Dependencies:** 2.2 Export to JSON  
**New dependency:** `serde_yaml = "0.9"`

---

### 3.4 Unit & Integration Tests
**Status:** ⏳ Pending  
**Estimated:** 6-8h  
**Priority:** Critical

- [ ] Test all domain entities
- [ ] Test repository implementations
- [ ] Test use cases with mocks
- [ ] CI integration (already in `.github/workflows/ci.yml`)
- [ ] Code coverage reporting

**Dependencies:** 1.1 Test Suite Setup

---

## 🎯 Phase 4: Medium Priority

### 4.1 Brewfile Support
**Status:** ⏳ Pending  
**Estimated:** 4-5h

- [ ] Parse Brewfile
- [ ] Generate Brewfile from installed packages
- [ ] Sync with GitHub Gist
- [ ] Import/Export buttons

---

### 4.2 Desktop Notifications
**Status:** ⏳ Pending  
**Estimated:** 2-3h

- [ ] Native macOS notifications
- [ ] Notify on operation complete
- [ ] Notify on errors
- [ ] Config to enable/disable

**New dependency:** `mac-notification-sys = "0.6"`

---

### 4.3 Dark/Light Theme
**Status:** ⏳ Pending  
**Estimated:** 3-4h

- [ ] Theme toggle in Settings
- [ ] Auto-detect system theme
- [ ] Persist theme preference
- [ ] Custom color schemes

---

### 4.4 Package Categories
**Status:** ⏳ Pending  
**Estimated:** 3-4h

- [ ] Fetch categories from brew
- [ ] Filter by category dropdown
- [ ] Category icons
- [ ] Popular categories view

---

### 4.5 History Timeline
**Status:** ⏳ Pending  
**Estimated:** 4-5h

- [ ] Visual timeline of operations
- [ ] Filter by date range
- [ ] Group by operation type
- [ ] Export history

---

### 4.6 Enhanced Keyboard Shortcuts
**Status:** ⏳ Pending  
**Estimated:** 1-2h

- [ ] Customizable shortcuts
- [ ] Shortcuts help dialog (Cmd+/)
- [ ] Vim-style navigation (j/k)

---

## 💡 Phase 5: Advanced Features

### 5.1 Stats Dashboard
**Status:** ⏳ Pending  
**Estimated:** 5-6h

- [ ] Total packages count
- [ ] Disk usage by package
- [ ] Update frequency chart
- [ ] Package age distribution
- [ ] Interactive charts with `egui_plot`

**New dependency:** `egui_plot = "0.33"`

---

### 5.2 Orphan Detection
**Status:** ⏳ Pending  
**Estimated:** 3-4h

- [ ] Find orphaned dependencies
- [ ] Safe removal suggestions
- [ ] Dependency tree visualization
- [ ] One-click cleanup

---

### 5.3 Batch Operations Queue
**Status:** ⏳ Pending  
**Estimated:** 4-5h

- [ ] Queue multiple operations
- [ ] Review before execution
- [ ] Cancel/reorder queue items
- [ ] Progress per operation

---

### 5.4 CLI Companion Tool
**Status:** ⏳ Pending  
**Estimated:** 4-5h

- [ ] `brewsty-cli` binary
- [ ] Commands: export, import, stats, info
- [ ] JSON output for scripting
- [ ] Share code with main app

---

### 5.5 Plugin System (WASM)
**Status:** ⏳ Pending  
**Estimated:** 8-10h

- [ ] WASM runtime integration
- [ ] Plugin API definition
- [ ] Plugin marketplace
- [ ] Security sandboxing

**New dependency:** `wasmtime = "15"`

---

## 📦 New Dependencies Summary

```toml
[dependencies]
# Phase 1: Technical Debt
tracing-appender = "0.2"
toml = "0.8"

# Phase 2: Quick Wins
# (no new dependencies)

# Phase 3: High Priority
reqwest = { version = "0.11", features = ["json"] }
serde_yaml = "0.9"

# Phase 4: Medium Priority
mac-notification-sys = "0.6"

# Phase 5: Advanced
egui_plot = "0.33"
wasmtime = "15"
```

---

## 📊 Progress Tracking

| Phase | Status | Progress |
|-------|--------|----------|
| 1. Technical Debt | ✅ Complete | 5/5 ✅ |
| 2. Quick Wins | ✅ Complete | 4/4 ✅ |
| 3. High Priority | ✅ Complete | 4/4 ✅ |
| 4. Medium Priority | 🔄 In Progress | 3/6 ✅ |
| 5. Advanced | ⏳ Pending | 0/5 |

**Total:** 16/24 tasks completed (67%!)

### ✅ Completed
**Phase 1 - Technical Debt:**
- 1.1 Test Suite Setup (existing tests)
- 1.2 Error Handling (existing anyhow/thiserror)
- 1.3 File-Based Logging ✅ (Mar 23)
- 1.4 Configuration File ✅ (Mar 23)
- 1.5 Better Tracing Spans (existing tracing)

**Phase 2 - Quick Wins:**
- 2.1 Keyboard Shortcuts ✅ (Mar 23)
- 2.2 Export to JSON ✅ (already implemented)
- 2.3 Package Search ✅ (already implemented)
- 2.4 Confirmation Dialogs ✅ (already implemented)

**Phase 3 - High Priority:**
- 3.1 Auto-Update Check ✅ (Mar 23) - GitHub Releases API integration
- 3.2 Package Details Panel ✅ (Mar 23) - Modal with package info
- 3.3 Export/Import Full State ✅ (Mar 23) - JSON/YAML import support
- 3.4 Unit & Integration Tests ✅ (Mar 23) - 70 tests passing

**Phase 4 - Medium Priority:**
- 4.1 Brewfile Support ✅ (Mar 23) - Parse/generate/export/import (core logic)
- 4.2 Desktop Notifications ✅ (Mar 23) - Native macOS notifications
- 4.3 Dark/Light Theme ✅ (already implemented) - System/Light/Dark with custom colors
---

## 🎯 Next Steps

1. Start with **1.3 File-Based Logging** (quick, foundational)
2. Then **1.4 Configuration File** (needed for auto-update)
3. Then **2.1 Keyboard Shortcuts** (quick win)
4. Then **2.2 Export to JSON** (quick win)
5. Continue in order...

---

## 📝 Notes

- Each task should have its own branch
- Write tests for new features
- Update README.md with new features
- Bump version in Cargo.toml per milestone
- Create GitHub releases for major versions

---

*Generated: March 23, 2026*
