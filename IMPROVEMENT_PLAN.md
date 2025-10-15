# Brusty - Architecture Improvement Plan

## Overview
This document outlines comprehensive improvements to enhance DI, components, DRY, SOLID principles, and code hierarchy.

## Current Issues

### 1. Dependency Injection (DI) Issues
**Problems:**
- ❌ 10 use case parameters in `BrustyApp::new()` - excessive constructor injection
- ❌ Runtime created in struct instead of injected
- ❌ Each use case creates threads/runtimes independently

**Impact:** Hard to test, maintain, and extend

### 2. DRY Violations
**Major Code Duplication:**
- `load_installed_packages()` vs `load_outdated_packages()` - **90% identical**
- `handle_install()`, `handle_uninstall()`, `handle_update()` - **similar pattern**
- Status message + logging pattern repeated ~20 times

**Impact:** Bug fixes need to be applied in multiple places, increased maintenance burden

### 3. SOLID Violations

#### Single Responsibility Principle (SRP):
- ❌ `BrustyApp` has **8+ responsibilities**:
  - UI rendering
  - Async task management
  - State management
  - Package operations
  - Logging
  - Filtering
  - Cleanup operations
  - Search functionality
- ✅ **Should be:** `BrustyApp` only coordinates, delegates to specialized components

#### Open/Closed Principle:
- ❌ Adding new tabs requires modifying `BrustyApp`
- ✅ **Should use:** Tab trait/registry pattern

#### Dependency Inversion:
- ❌ `AsyncTask` enum tightly coupled to specific data structures
- ✅ **Should use:** Generic task abstraction

### 4. Component Architecture Issues

**Missing Components:**
```
src/presentation/
  components/
    ✅ package_list.rs
    ❌ async_task_manager.rs    (NEW)
    ❌ package_operation_handler.rs (NEW)
    ❌ tab_manager.rs           (NEW)
    ❌ filter_state.rs          (NEW)
    ❌ cleanup_modal.rs         (NEW)
    ❌ status_bar.rs            (NEW)
    ❌ output_log.rs            (NEW)
  services/
    ❌ async_executor.rs        (NEW)
    ❌ event_bus.rs             (NEW)
```

### 5. Hierarchy Issues

**Flattened Use Case Structure:**
```
use_cases/
  ❌ All 8+ use cases in one file (package_operations.rs)
  
✅ Should be organized by domain:
  use_cases/
    queries/
      list_installed.rs
      list_outdated.rs
      search_packages.rs
      get_package_info.rs
    commands/
      install_package.rs
      uninstall_package.rs
      update_package.rs
      update_all_packages.rs
    maintenance/
      clean_cache.rs
      cleanup_old_versions.rs
```

## Implementation Plan

### Phase 1: HIGH PRIORITY (Biggest Impact)

#### 1. Create UseCaseContainer
**File:** `src/application/use_case_container.rs`

Consolidates all use cases into a single injectable container:
```rust
pub struct UseCaseContainer {
    pub list_installed: Arc<ListInstalledPackages>,
    pub list_outdated: Arc<ListOutdatedPackages>,
    pub install: Arc<InstallPackage>,
    pub uninstall: Arc<UninstallPackage>,
    pub update: Arc<UpdatePackage>,
    pub update_all: Arc<UpdateAllPackages>,
    pub clean_cache: Arc<CleanCache>,
    pub cleanup_old_versions: Arc<CleanupOldVersions>,
    pub search: Arc<SearchPackages>,
    pub get_package_info: Arc<GetPackageInfo>,
}
```

**Benefits:**
- Reduces `BrustyApp::new()` from 10 parameters to 1
- Easier to add new use cases
- Better encapsulation

#### 2. Create AsyncExecutor Service
**File:** `src/presentation/services/async_executor.rs`

Manages runtime lifecycle and async task execution:
```rust
pub struct AsyncExecutor {
    runtime: tokio::runtime::Runtime,
}

impl AsyncExecutor {
    pub fn execute<F, T>(&self, future: F) -> T
    where F: Future<Output = T>;
    
    pub fn spawn<F>(&self, future: F) -> JoinHandle<F::Output>;
}
```

**Benefits:**
- Single runtime instance
- Centralized async execution
- Better resource management
- Testable async code

#### 3. Create AsyncTaskManager Component
**File:** `src/presentation/components/async_task_manager.rs`

Separates async task polling and management from UI:
```rust
pub struct AsyncTaskManager {
    active_task: Option<AsyncTask>,
    package_info_tasks: Vec<(String, AsyncTask)>,
    packages_loading_info: HashSet<String>,
    pending_loads: Vec<(String, PackageType)>,
}

impl AsyncTaskManager {
    pub fn poll_tasks(&mut self, /* callbacks */);
    pub fn start_package_load(&mut self, name: String, type: PackageType);
    pub fn is_loading(&self, package_name: &str) -> bool;
}
```

**Benefits:**
- Separation of concerns
- Reusable async task logic
- Easier testing
- Reduces `app.rs` complexity

#### 4. Create PackageOperationHandler
**File:** `src/presentation/services/package_operation_handler.rs`

Eliminates DRY violations in package operations:
```rust
pub struct PackageOperationHandler {
    use_cases: Arc<UseCaseContainer>,
    executor: Arc<AsyncExecutor>,
}

impl PackageOperationHandler {
    pub async fn handle_operation(
        &self,
        operation: PackageOperation,
        package: Package,
    ) -> Result<OperationResult>;
}

pub enum PackageOperation {
    Install,
    Uninstall,
    Update,
}

pub struct OperationResult {
    pub success: bool,
    pub message: String,
    pub should_reload_installed: bool,
    pub should_reload_outdated: bool,
}
```

**Benefits:**
- DRY - Single place for operation logic
- Consistent error handling
- Easier to add new operations
- Unified logging pattern

#### 5. Extract StatusNotifier/EventBus
**File:** `src/presentation/services/event_bus.rs`

Decouples status updates and logging:
```rust
pub struct EventBus {
    status_message: Arc<Mutex<String>>,
    output_log: Arc<Mutex<Vec<String>>>,
}

impl EventBus {
    pub fn notify_status(&self, message: String);
    pub fn log(&self, message: String);
    pub fn get_status(&self) -> String;
    pub fn get_recent_logs(&self, count: usize) -> Vec<String>;
}
```

**Benefits:**
- Decoupled logging
- Thread-safe status updates
- Observable pattern
- Better for testing

### Phase 2: MEDIUM PRIORITY (Better Architecture)

#### 6. Create TabManager Component
**File:** `src/presentation/components/tab_manager.rs`

Manages tab state and navigation:
```rust
pub struct TabManager {
    current_tab: Tab,
    tab_states: HashMap<Tab, TabState>,
}

impl TabManager {
    pub fn switch_to(&mut self, tab: Tab);
    pub fn current(&self) -> &Tab;
    pub fn is_loaded(&self, tab: Tab) -> bool;
    pub fn mark_loaded(&mut self, tab: Tab);
}
```

**Benefits:**
- Centralized tab logic
- Easier to add new tabs
- Cleaner state management

#### 7. Create FilterState Component
**File:** `src/presentation/components/filter_state.rs`

Manages filtering state:
```rust
pub struct FilterState {
    show_formulae: bool,
    show_casks: bool,
    search_query: String,
    installed_search_query: String,
}

impl FilterState {
    pub fn should_show_package(&self, package: &Package, context: FilterContext) -> bool;
    pub fn reset(&mut self);
}
```

**Benefits:**
- Reusable filter logic
- Centralized filter state
- Easier to add new filters

#### 8. Reorganize Use Case Hierarchy
**Structure:**
```
src/application/use_cases/
  queries/
    list_installed.rs
    list_outdated.rs
    search_packages.rs
    get_package_info.rs
  commands/
    install_package.rs
    uninstall_package.rs
    update_package.rs
    update_all_packages.rs
  maintenance/
    clean_cache.rs
    cleanup_old_versions.rs
  mod.rs (re-exports)
```

**Benefits:**
- Clear CQRS pattern
- Better organization
- Easier to navigate
- Scalable structure

#### 9. Add Proper Error Types
**File:** `src/domain/errors.rs`

Custom error types for better error handling:
```rust
#[derive(Debug, thiserror::Error)]
pub enum BrustyError {
    #[error("Package not found: {0}")]
    PackageNotFound(String),
    
    #[error("Installation failed: {0}")]
    InstallationFailed(String),
    
    #[error("Brew command failed: {0}")]
    BrewCommandFailed(String),
    
    #[error("Timeout loading package info: {0}")]
    PackageInfoTimeout(String),
}
```

**Benefits:**
- Type-safe error handling
- Better error messages
- Easier debugging
- Pattern matching on errors

### Phase 3: LOW PRIORITY (Nice to Have)

#### 10. Extract CleanupModalComponent
**File:** `src/presentation/components/cleanup_modal.rs`

Separate cleanup modal UI logic:
```rust
pub struct CleanupModal {
    show: bool,
    cleanup_type: Option<CleanupType>,
    preview: Option<CleanupPreview>,
}

impl CleanupModal {
    pub fn render(&mut self, ctx: &egui::Context) -> Option<CleanupAction>;
}
```

#### 11. Create LogManager Component
**File:** `src/presentation/components/log_manager.rs`

Manages output log with features:
```rust
pub struct LogManager {
    logs: VecDeque<LogEntry>,
    max_size: usize,
    filters: Vec<LogFilter>,
}

impl LogManager {
    pub fn push(&mut self, message: String, level: LogLevel);
    pub fn get_recent(&self, count: usize) -> Vec<&LogEntry>;
    pub fn filter(&self, filter: LogFilter) -> Vec<&LogEntry>;
}
```

#### 12. Create StatusBar Component
**File:** `src/presentation/components/status_bar.rs`

Dedicated status bar component:
```rust
pub struct StatusBar {
    message: String,
    loading: bool,
}

impl StatusBar {
    pub fn render(&self, ui: &mut egui::Ui);
}
```

## Expected Outcomes

### Code Metrics (Estimated)
- **app.rs**: 1115 lines → ~400 lines (64% reduction)
- **Cyclomatic Complexity**: Reduced by ~60%
- **Test Coverage**: Easier to achieve >80%
- **Parameters in constructors**: 10 → 3-4

### Architecture Improvements
- ✅ Clear separation of concerns
- ✅ Testable components
- ✅ Reusable services
- ✅ SOLID compliance
- ✅ DRY code
- ✅ Better DI

### Maintainability
- ✅ New features easier to add
- ✅ Bugs easier to locate and fix
- ✅ Code easier to understand
- ✅ Better onboarding for new developers

## Implementation Order

1. **Day 1: Foundation (HIGH)**
   - UseCaseContainer (#1)
   - AsyncExecutor (#2)
   - AsyncTaskManager (#3)

2. **Day 2: Core Logic (HIGH)**
   - PackageOperationHandler (#5)
   - EventBus (#4)

3. **Day 3: Component Extraction (MEDIUM)**
   - TabManager (#6)
   - FilterState (#7)
   - Use Case Reorganization (#8)

4. **Day 4: Polish (MEDIUM/LOW)**
   - Error Types (#9)
   - CleanupModal (#10)
   - LogManager (#11)
   - StatusBar (#12)

## Testing Strategy

Each new component should have:
- ✅ Unit tests
- ✅ Integration tests (where applicable)
- ✅ Mock implementations for repositories
- ✅ Property-based tests for complex logic

## Migration Path

To minimize risk:
1. Create new components alongside existing code
2. Gradually migrate functionality
3. Run tests after each migration
4. Remove old code only when fully migrated
5. Update documentation

## Success Criteria

- [ ] All tests passing
- [ ] No clippy warnings
- [ ] app.rs < 500 lines
- [ ] All components have unit tests
- [ ] No code duplication for common operations
- [ ] Constructor injection ≤ 4 parameters
- [ ] Clear component boundaries
- [ ] Documentation updated
