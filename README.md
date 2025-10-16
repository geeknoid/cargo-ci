# cargo-ci

[![crate.io](https://img.shields.io/crates/v/cargo-ci.svg)](https://crates.io/crates/cargo-ci)
[![docs.rs](https://docs.rs/cargo-ci/badge.svg)](https://docs.rs/cargo-ci)
[![CI](https://github.com/geeknoid/cargo-ci/workflows/main/badge.svg)](https://github.com/geeknoid/cargo-ci/actions)
[![Coverage](https://codecov.io/gh/geeknoid/cargo-ci/graph/badge.svg?token=FCUG0EL5TI)](https://codecov.io/gh/geeknoid/cargo-ci)
[![Minimum Supported Rust Version 1.88](https://img.shields.io/badge/MSRV-1.88-blue.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

* [Summary](#summary)
* [Configuration](#configuration)
  * [Workspace Configuration (`ci.toml`)](#workspace-configuration-citoml)
  * [Job Configuration](#job-configuration)
  * [Step Configuration](#step-configuration)
  * [Package Configuration (`Cargo.toml`)](#package-configuration-cargotoml)
* [Variables and Conditional Execution](#variables-and-conditional-execution)
* [Tool Management](#tool-management)
* [Command-Line Interface](#command-line-interface)
  * [Running Jobs](#running-jobs)
  * [Other Commands](#other-commands)
* [Examples](#examples)

## Summary

<!-- cargo-rdme start -->

A lightweight tool for running CI-style job pipelines locally across a Cargo workspace.

`cargo-ci` lets you define CI workflows in a `ci.toml` file and execute them on your local machine,
providing a consistent and reproducible way to run checks, builds, and tests across all crates in your workspace.

## Key Features

* **Job Orchestration**: Define jobs with steps and dependencies in a `ci.toml` file
* **Dependency Management**: Jobs can depend on other jobs via the `needs` field
* **Conditional Execution**: Control job and step execution with boolean expressions
* **Variable System**: Define and use variables at workspace and package levels
* **Tool Management**: Automatically install required cargo tools
* **Package Selection**: Run jobs on specific packages or the entire workspace

## Quick Start

Create a `ci.toml` file in your workspace root:

```toml
[jobs.build]
steps = ["cargo build --workspace"]

[jobs.test]
needs = ["build"]
steps = ["cargo test --workspace"]
```

Run your jobs:

```bash
# List all jobs
cargo ci list-jobs

# Run specific jobs
cargo ci run build test

# Run all jobs
cargo ci run -a
```

## Configuration

### Workspace Configuration (`ci.toml`)

The `ci.toml` file defines jobs, tools, and variables for your workspace:

```toml
# Define workspace-level variables
variables = ["BUILD_MODE", "TARGET_ARCH"]

# Specify required tools and versions
[tools]
cargo-rdme = "1.5.0"
cargo-nextest = "0.9.0"

[jobs.checkout]
name = "Checkout Code"
steps = ["git clone https://example.com/repo.git"]

[jobs.build]
name = "Build Project"
needs = ["checkout"]
cond = "BUILD_MODE"
steps = [
  "cargo build --workspace --all-targets --release"
]
```

### Job Configuration

Jobs can have the following fields:

* `steps` (required): Array of commands to execute
* `needs` (optional): Array of job IDs that must complete successfully first
* `name` (optional): Human-readable display name for the job
* `cond` (optional): Boolean expression that must be true for the job to run
* `keep_going` (optional): If `true`, continue even if steps fail

### Step Configuration

Steps can be simple command strings or extended configuration objects:

```toml
[jobs.test]
steps = [
  # Simple step
  "cargo test --workspace",

  # Extended step with options
  { command = "cargo test --doc", name = "Doc tests", cond = "ENABLE_DOC_TESTS" },

  # Per-package step
  { command = "cargo check", per_package = true },

  # Step that can fail without stopping the job
  { command = "cargo bench", keep_going = true }
]
```

#### Extended Step Fields

* `command` (required): The command to execute
* `name` (optional): Display name for the step
* `cond` (optional): Boolean expression to conditionally execute the step
* `per_package` (optional): If `true`, run the command once per package
* `keep_going` (optional): If `true`, continue even if this step fails

### Package Configuration (`Cargo.toml`)

Packages can define variables in their `Cargo.toml`:

```toml
[package.metadata.ci.variables]
BUILD_MODE = "release"
ENABLE_DOC_TESTS = "true"
```

## Variables and Conditional Execution

Variables can be defined at three levels:

1. **Workspace level**: Listed in `ci.toml` under `variables`
2. **Package level**: Defined in `Cargo.toml` under `[package.metadata.ci.variables]`
3. **Command line**: Passed via `-v VAR=VALUE`

Boolean expressions support:
* `&` (AND): `BUILD_MODE & RELEASE`
* `|` (OR): `WINDOWS | MACOS`
* `!` (NOT): `!DEBUG_MODE`
* `(...)` (grouping): `(WINDOWS | MACOS) & !DEBUG`

Variables are evaluated as true if they are set (present in the environment or defined).

## Tool Management

Define required tools in `ci.toml`:

```toml
[tools]
cargo-nextest = "0.9.0"
cargo-rdme = "1.5.0"
```

Install them with:

```bash
cargo ci install
```

## Command-Line Interface

### Running Jobs

```bash
# Run specific jobs
cargo ci run build test

# Run all jobs
cargo ci run -a

# Run jobs for specific packages
cargo ci run -p my-package -p other-package test

# Dry run (show what would be executed)
cargo ci run -n -a

# Keep going after failures
cargo ci run -k test

# Pass variables
cargo ci run -v BUILD_MODE=debug -v ENABLE_TESTS=true build
```

### Other Commands

```bash
# List all defined jobs
cargo ci list-jobs

# Install required tools
cargo ci install
```

## Examples

### Complete Example

**`ci.toml`**:
```toml
variables = ["NIGHTLY", "COVERAGE"]

[tools]
cargo-nextest = "0.9.0"
cargo-tarpaulin = "0.27.0"

[jobs.build]
name = "Build All Targets"
steps = [
  "cargo build --workspace --all-targets"
]

[jobs.test]
name = "Run Tests"
needs = ["build"]
steps = [
  { command = "cargo nextest run --workspace", name = "Unit tests" },
  { command = "cargo test --doc", name = "Doc tests" }
]

[jobs.lint]
name = "Lint Code"
steps = [
  { command = "cargo clippy --all-targets -- -D warnings", name = "Clippy" },
  { command = "cargo fmt --check", name = "Format check" }
]

[jobs.coverage]
name = "Code Coverage"
needs = ["test"]
cond = "COVERAGE"
steps = [
  "cargo tarpaulin --workspace --out Xml"
]
```

**`Cargo.toml`** (in a package):
```toml
[package.metadata.ci.variables]
NIGHTLY = "true"
```

Run it:
```bash
cargo ci install
cargo ci run -a
cargo ci run -v COVERAGE=true coverage
```

<!-- cargo-rdme end -->
