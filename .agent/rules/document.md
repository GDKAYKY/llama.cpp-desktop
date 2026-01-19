---
trigger: always_on
---

# System Prompt – Project Rules

## Language & Documentation
- All comments, documentation, and identifiers MUST be written in EN-US.
- No mixed languages. Ever.
- All documentation MUST live under the `/docs` directory.
- Each major feature MUST have its own markdown file in `/docs`.

## Documentation Standards
- Every non-trivial feature MUST include:
  - Purpose
  - Architecture overview
  - Responsibilities of each module
  - Public interfaces
- Documentation MUST be updated together with code changes.

## Separation of Responsibilities
- Enforce clear boundaries:
  - UI logic
  - Business logic
  - Infrastructure / IO
- No module should handle more than one responsibility.
- Cross-layer access is forbidden (UI → Service → Infrastructure only).

## Code Quality
- Prefer official documentation over assumptions or conventions.
- Before implementing a feature, verify best practices in:
  - Official framework documentation
  - Well-established community standards
- Avoid custom abstractions unless strictly necessary.

## Configuration Management
- All configuration MUST be defined in JSON files.
- No hardcoded configuration values inside source code.
- Environment-specific configuration MUST be isolated and explicit.

## Testing
- All features MUST be covered by tests written in Jest.
- Focus on:
  - Unit tests for business logic
  - Integration tests for feature behavior
- Tests MUST be readable, deterministic, and isolated.
- No untested critical logic is allowed.

## General Rules
- Clarity > cleverness
- Predictability > flexibility
- Explicit > implicit