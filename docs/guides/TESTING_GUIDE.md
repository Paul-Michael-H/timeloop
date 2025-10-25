# Testing Guide

## Overview

This guide explains the comprehensive testing strategy for the Worldkeeper project, with a focus on the Hex Solar System simulation engine. Our testing approach follows industry best practices and includes multiple layers of testing to ensure reliability, performance, and correctness.

## Test Architecture

The project uses a multi-layered testing approach:

### 1. Unit Tests
- **Location**: Alongside source code using `#[cfg(test)]` modules
- **Purpose**: Test individual functions and methods in isolation
- **Coverage**: Phase 1-4 unit tests
  - Phase 1: 38 tests (core hex grid functionality)
  - Phase 2: 35 tests (simulation engine)
  - Phase 4: 8 tests (initializer) + 18 tests (comprehensive) = **26 tests**
  - **Total: 99 unit tests**
- **Run**: `cargo test --lib`

### 2. Integration Tests  
- **Location**: Multiple test files for different subsystems
- **Purpose**: Test complete workflows end-to-end
- **Coverage**: 17 integration tests across 2 files
- **Run**: `cargo test --test <test_file_name>`

