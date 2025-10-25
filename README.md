# Worldkeeper Backend Service

A minimal Rust backend service with health check endpoints built using Axum.

## 🦀 RUST-ONLY SERVER POLICY

**⚠️ CRITICAL**: This project uses **ONLY Rust web services**. No Node.js, Python, or other language servers permitted.


## 📚 Documentation

Comprehensive project documentation is available in the [`docs/`](docs/) directory:

DO NOT CHANGE THESE DOCUMENTS UNLESS ASKED TO DO SO EXPLICITLY.
ALWAYS
  place progress reports, final reports and assessments into
  ./docs/workplace
  place all plans in
  ./docs/plans

- **[Documentation Index](docs/README.md)** - Complete navigation guide
- **[Server Guidelines](docs/SERVER_GUIDELINES.md)** - 🦀 **Rust-only server requirements**
- **[Code Quality Standards](docs/guides/CODE_QUALITY_STANDARDS.md)** - Coding conventions and best practices
- **[Testing Guide](docs/guides/TESTING_GUIDE.md)** - Testing patterns and TDD approach
- **[Architecture Overview](docs/architecture/)** - System design and technical architecture
- **[API Documentation](docs/api/)** - API specifications and response formats

## Features

- ✅ Health check endpoint at `/health` and `/api/health`
- ✅ CORS enabled for cross-origin requests
- ✅ JSON responses with service information
- ✅ Structured logging with tracing
- ✅ Async/await with Tokio runtime
- ✅ 100% test coverage (509/509 tests passing)

## Dependencies

- **axum**: Modern async web framework for Rust
- **tokio**: Async runtime
- **serde**: Serialization/deserialization 
- **tower-http**: HTTP middleware (CORS)
- **tracing**: Structured logging
- **chrono**: Date/time handling

## Running the Service

```bash
# Build the project
cargo build

# Run the service
cargo run
```


## Testing

Run the test suite:
```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test module
cargo test storage::async_loader::tests
```

