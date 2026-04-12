# Test Suite Documentation

## Overview

This test suite provides comprehensive coverage (80%+) for the Llama Desktop application, covering both Rust backend and TypeScript/Svelte frontend.

## Structure

### Backend Tests (`src-tauri/tests/`)

```
tests/
├── common/
│   └── mod.rs              # Shared helpers and fixtures
├── models_test.rs          # Model serialization tests
├── utils_test.rs           # Utility function tests
├── parsing_test.rs         # Model path parsing tests
├── infrastructure/
│   ├── mod.rs
│   ├── process_manager_test.rs
│   ├── metrics_test.rs
│   ├── nvidia_smi_test.rs
│   └── llama_server_test.rs
├── services/
│   ├── mod.rs
│   ├── llama_service_test.rs
│   ├── mcp_service_test.rs
│   ├── orchestrator_test.rs
│   ├── capability_registry_test.rs
│   └── thinking_parser_test.rs
├── commands/
│   ├── mod.rs
│   ├── config_test.rs
│   ├── mcp_config_test.rs
│   └── models_test.rs
└── integration/
    ├── mod.rs
    ├── model_lifecycle_test.rs
    ├── mcp_lifecycle_test.rs
    └── chat_flow_test.rs
```

### Frontend Tests (`tests/`)

```
tests/
├── setup.ts                # Test setup and configuration
├── stores/
│   ├── models.test.ts
│   ├── server.test.ts
│   ├── settings.test.ts
│   ├── mcp.test.ts
│   └── ui.test.ts
├── services/
│   ├── history.test.ts
│   ├── models.test.ts
│   ├── models_delete.test.ts
│   ├── model_downloads.test.ts
│   ├── chat_templates.test.ts
│   └── orchestrator.test.ts
└── config/
    └── index.test.ts
```

## Running Tests

### Backend (Rust)

```bash
cd src-tauri
cargo test                    # Run all tests
cargo test --test integration # Run specific test file
cargo test models_test        # Run specific module
```

### Frontend (TypeScript/Svelte)

```bash
npm test                      # Run all tests
npm test -- --run             # Run without watch mode
npm test -- --coverage        # Generate coverage report
```

### Coverage Reports

#### Rust

```bash
cd src-tauri
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

#### Frontend

```bash
npm test -- --coverage
# Report in coverage/index.html
```

## Test Strategy

### Unit Tests

- **Models**: Serialization/deserialization, validation
- **Utils**: File I/O, JSON operations
- **Services**: Business logic with mocked dependencies
- **Commands**: Input validation, error handling

### Integration Tests

- **Model Lifecycle**: Start → configure → stop
- **MCP Lifecycle**: Connect → call tools → disconnect
- **Chat Flow**: Session management, message handling

### Mocking Strategy

- **Unit tests**: Mock all external dependencies (filesystem, HTTP, processes)
- **Integration tests**: Use real filesystem (tempdir), mock only external services

## Coverage Thresholds

- **Backend (Rust)**: 80% minimum
- **Frontend (TypeScript)**: 85% minimum (configured in vite.config.js)

## CI/CD

Tests run automatically on:

- Push to `main` branch
- Pull requests to `main`
- Release branches

Coverage reports are uploaded to:

- Codecov
- SonarCloud

## Common Patterns

### Creating Test Fixtures (Rust)

```rust
use crate::common;

let config = common::sample_llama_config();
let server = common::sample_mcp_server("test-id");
let message = common::sample_chat_message("user", "Hello");
```

### Mocking Tauri IPC (Frontend)

```typescript
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));

const { invoke } = await import("@tauri-apps/api/core");
vi.mocked(invoke).mockResolvedValue(expectedResult);
```

### Using Temporary Directories (Rust)

```rust
let dir = common::temp_dir();
let path = dir.path().join("test.json");
// Directory is automatically cleaned up when `dir` is dropped
```

## Maintenance

- Keep tests close to the code they test
- Update tests when changing APIs
- Maintain 80%+ coverage on all new code
- Run tests before committing
