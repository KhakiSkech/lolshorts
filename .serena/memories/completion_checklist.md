# LoLShorts Task Completion Checklist

## Every Task Must Complete These Steps

### 1. Code Quality
- [ ] **Format code**
  - Rust: `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
  - TypeScript: `npm run format`

- [ ] **Lint code**
  - Rust: `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
  - TypeScript: `npm run lint`

- [ ] **No compiler warnings**
  - Rust: `cargo build --manifest-path src-tauri/Cargo.toml` (0 warnings)
  - TypeScript: `npm run build` (0 errors)

### 2. Testing
- [ ] **Write tests FIRST** (TDD)
  - Unit tests for new functions
  - Integration tests for features
  - Edge case coverage

- [ ] **Run all tests**
  - Backend: `cargo test --manifest-path src-tauri/Cargo.toml`
  - Frontend: `npm test`
  - All tests must pass (no skipped or ignored tests)

- [ ] **Test coverage**
  - Unit tests: >80% coverage
  - Integration tests: Critical paths covered

### 3. Documentation
- [ ] **Code documentation**
  - Public functions have doc comments (`///` in Rust, JSDoc in TypeScript)
  - Complex logic has explanatory comments (WHY, not WHAT)
  - Non-obvious decisions documented

- [ ] **Update README/docs** (if applicable)
  - Feature documentation
  - API changes
  - Breaking changes

### 4. Security
- [ ] **Input validation**
  - All user inputs validated
  - Path traversal prevention (no `..` in paths)
  - SQL injection prevention (use parameterized queries)

- [ ] **No secrets in code**
  - No hardcoded passwords, tokens, API keys
  - Sensitive data stored securely (Windows Credential Manager)
  - No secrets in logs

- [ ] **Error handling**
  - No `.unwrap()` or `.expect()` in production code
  - Use `?` operator for error propagation
  - Meaningful error messages (no generic "Error occurred")

### 5. Performance
- [ ] **Meet performance targets**
  - App startup: <3s cold start
  - LCU connection: <2s
  - Event detection: <500ms latency
  - Video processing: <30s per minute of footage

- [ ] **Resource usage**
  - Memory: <500MB idle, <2GB during processing
  - CPU: Reasonable utilization
  - No memory leaks (check with profiler)

### 6. Git Workflow
- [ ] **Commit message format**
  ```
  <type>(<scope>): <subject>

  <body>

  <footer>
  ```
  - Types: feat, fix, refactor, test, docs, perf, chore
  - Scope: module name (recording, video, auth, etc.)
  - Subject: concise description (imperative mood)

- [ ] **Clean commit history**
  - Logical commits (not "fix typo" × 10)
  - Each commit compiles and tests pass

### 7. Feature Completeness
- [ ] **No partial implementations**
  - No TODO comments for core functionality
  - No mock objects or stub implementations
  - No "Not implemented" errors

- [ ] **Full feature scope**
  - All requirements implemented
  - Edge cases handled
  - Error cases handled

### 8. Validation
- [ ] **Manual testing**
  - Feature works as expected
  - UI is responsive and intuitive
  - No visual bugs

- [ ] **Integration testing**
  - Feature integrates with existing code
  - No breaking changes to other features
  - Backward compatibility (if applicable)

## Before Marking Task as Complete

### Final Checklist
```bash
# 1. Format
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
npm run format

# 2. Lint
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
npm run lint

# 3. Build
cargo build --manifest-path src-tauri/Cargo.toml
npm run build

# 4. Test
cargo test --manifest-path src-tauri/Cargo.toml
npm test

# 5. Verify no warnings/errors
# All commands above should exit with 0 warnings/errors

# 6. Manual test
cargo tauri dev
# Test feature manually in running app

# 7. Commit
git add .
git commit -m "feat(module): description"

# 8. Update todo list
# Mark task as completed in TodoWrite
```

## Quality Gates by Wave

### Wave 1: LCU Integration & Authentication
- [ ] Supabase authentication working (login, logout, session refresh)
- [ ] License tier enforcement (FREE/PRO feature gating)
- [ ] Screenshot capture functional (750×422 resolution)
- [ ] Enhanced event detection (priorities 1-5)
- [ ] All unit tests pass
- [ ] Manual testing successful

### Wave 2: Video Processing Pipeline
- [ ] FFmpeg wrapper operational (extract, compose, thumbnails)
- [ ] Clip extraction working (<5s for 10s clip)
- [ ] Multi-clip composition with transitions
- [ ] Audio normalization (LUFS -16)
- [ ] Performance benchmarks met

### Wave 3: Auto-Composition & AI
- [ ] Composition engine functional (Fast/Balanced/Cinematic)
- [ ] Beat detection working (aubio integration)
- [ ] Template system implemented
- [ ] AI clip scorer trained and deployed
- [ ] Music sync validation

### Wave 4: Professional Editor UI
- [ ] Clip gallery with filtering/sorting
- [ ] Timeline editor with drag-and-drop
- [ ] Transition controls working
- [ ] Export settings functional
- [ ] UI/UX validated

### Wave 5: Deployment & Distribution
- [ ] Tauri updater configured
- [ ] CI/CD pipeline working
- [ ] Code signing successful
- [ ] Auto-update tested
- [ ] Beta deployment successful

## Remember
- **Quality over speed**: Better to do it right than do it fast
- **Test first**: TDD prevents bugs and improves design
- **Evidence-based**: Measure performance, don't assume
- **Security first**: Never compromise on security fundamentals