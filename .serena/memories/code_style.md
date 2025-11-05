# LoLShorts Code Style & Conventions

## Rust Backend

### Naming Conventions
- **Structs and Enums**: PascalCase
  - `GameDvrController`, `LicenseTier`, `RecordingStatus`
- **Functions and Variables**: snake_case
  - `start_recording()`, `get_user_status()`, `current_game`
- **Constants**: SCREAMING_SNAKE_CASE
  - `MAX_CLIPS_PER_GAME`, `DEFAULT_BUFFER_SIZE`
- **Lifetimes**: lowercase single letter or descriptive
  - `'a`, `'static`, `'env`

### Module Organization
- **Flat module structure** in src-tauri/src/
- **Re-export commonly used types** from mod.rs
- **Commands module** for Tauri-exposed functions
- **Models module** for data structures

### Error Handling
- **Custom errors** using `thiserror::Error`
- **Use `?` operator** for error propagation
- **NEVER use `.unwrap()` or `.expect()`** in production code
- **Return `Result<T, String>`** for Tauri commands (convert errors to strings)

### Async Patterns
- **Use `tokio::spawn`** for concurrent tasks
- **Minimal lock scopes** - drop locks before `.await`
- **Avoid blocking in async** - use `tokio::time::sleep` not `std::thread::sleep`

### Testing
- **Unit tests** in `#[cfg(test)] mod tests`
- **Helper functions at top** of test module
- **Group tests** by functionality using nested modules
- **Use `#[tokio::test]`** for async tests

## TypeScript/React Frontend

### Naming Conventions
- **Components**: PascalCase
  - `Dashboard`, `ClipCard`, `TimelineEditor`
- **Functions and Variables**: camelCase
  - `handleGameClick`, `isRecording`, `currentGame`
- **Types and Interfaces**: PascalCase
  - `User`, `GameEvent`, `RecordingStatus`
- **Constants**: SCREAMING_SNAKE_CASE
  - `MAX_CLIPS`, `DEFAULT_DURATION`

### Component Structure
1. **Props interface** at top
2. **State hooks** (useState, useEffect)
3. **Custom hooks**
4. **Event handlers** (useCallback)
5. **Helper functions**
6. **Render** (JSX return)

### React Hooks
- **Always include dependencies** in useEffect, useCallback, useMemo
- **Clean up effects** (return cleanup function)
- **Use custom hooks** to encapsulate reusable logic
- **Limit memoization** - only for expensive calculations

### State Management (Zustand)
- **Store structure**: State + Actions
- **Selectors** for derived state
- **Async actions** in store methods
- **Minimal subscriptions** (use selectors to prevent re-renders)

### Tauri Integration
- **Type-safe invocations** using TypeScript generics
- **Wrap commands in hooks** for state management
- **Handle errors** with try-catch blocks
- **Clean up event listeners** in useEffect cleanup

## General Principles
- **DRY**: Don't Repeat Yourself
- **KISS**: Keep It Simple, Stupid
- **YAGNI**: You Aren't Gonna Need It
- **SOLID**: Single Responsibility, Open/Closed, Liskov Substitution, Interface Segregation, Dependency Inversion
- **Test-First**: Write tests before implementation (TDD)
- **Evidence-Based**: Measure before optimizing
- **Security-First**: Validate all inputs, never log secrets

## Documentation
- **Function documentation** using doc comments (`///` in Rust, JSDoc in TypeScript)
- **Explain WHY, not WHAT** in comments
- **No obvious comments** (let code speak for itself)
- **Document non-obvious decisions** and trade-offs

## File Organization
- **Backend**: `src-tauri/src/<module>/{mod.rs, commands.rs, models.rs}`
- **Frontend**: `src/<feature>/{Component.tsx, hooks.ts, types.ts}`
- **Tests**: `src-tauri/tests/<feature>_test.rs` or inline `#[cfg(test)] mod tests`
- **Assets**: `src/assets/<type>/` (images, icons, etc.)