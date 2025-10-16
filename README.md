# cargo-ci

[![crate.io](https://img.shields.io/crates/v/cargo-ci.svg)](https://crates.io/crates/cargo-ci)
[![docs.rs](https://docs.rs/cargo-ci/badge.svg)](https://docs.rs/cargo-ci)
[![CI](https://github.com/geeknoid/cargo-ci/workflows/main/badge.svg)](https://github.com/geeknoid/cargo-ci/actions)
[![Coverage](https://codecov.io/gh/geeknoid/cargo-ci/graph/badge.svg?token=FCUG0EL5TI)](https://codecov.io/gh/geeknoid/cargo-ci)
[![Minimum Supported Rust Version 1.87](https://img.shields.io/badge/MSRV-1.87-blue.svg)]()
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](./LICENSE)

* [Summary](#summary)
* [Installation](#installation)
* [Configuration](#configuration)
  * [Option 1: Using `ci.toml` (Recommended)](#option-1-using-citoml-recommended)
  * [Option 2: Using `Cargo.toml` metadata](#option-2-using-cargotoml-metadata)
* [Usage](#usage)
* [Workspace Behavior](#workspace-behavior)
* [Custom Display Names](#custom-display-names)
  * [Job-level custom name](#job-level-custom-name)
  * [Step-level custom name](#step-level-custom-name)
* [Modifiers](#modifiers)
  * [Defining Project Modifiers](#defining-project-modifiers)
  * [Using Modifiers in Steps](#using-modifiers-in-steps)
  * [Modifier Expression Syntax](#modifier-expression-syntax)
  * [Examples](#examples)
  * [Operator Precedence](#operator-precedence)
  * [Behavior](#behavior)
  * [Use Cases](#use-cases)
* [Environment Variables](#environment-variables)

## Summary

<!-- cargo-rdme start -->

Simulate CI jobs locally by running configured commands across crates in a Cargo workspace.

## Installation

From the project directory:

```sh
cargo install cargo-ci
```

Then invoke via Cargo's plugin mechanism:

```sh
cargo ci --list
```

## Configuration

Jobs can be defined in two ways:

### Option 1: Using `ci.toml` (Recommended)

Create a `ci.toml` file at the root of your workspace with a simplified structure:

```toml
[jobs.build]
steps = ["cargo build --all --all-targets"]

[jobs.test]
steps = ["cargo test --all --all-features"]

[jobs.lints]
steps = [
  "cargo fmt -- --check",
  "cargo clippy --all -- -D warnings"
]
```

### Option 2: Using `Cargo.toml` metadata

Alternatively, jobs can be defined in `Cargo.toml` under either `[package.metadata.ci.jobs]`
(single crate) or `[workspace.metadata.ci.jobs]` (workspace root):

```toml
[workspace.metadata.ci.jobs.build]
steps = ["cargo build --all --all-targets"]

[workspace.metadata.ci.jobs.test]
steps = ["cargo test --all --all-features"]

[workspace.metadata.ci.jobs.lints]
steps = [
  "cargo fmt -- --check",
  "cargo clippy --all -- -D warnings"
]
```

**Note**: If `ci.toml` exists, it will be used instead of the `Cargo.toml` configuration.
Project-level settings (like modifiers) must always be defined in `Cargo.toml`.

## Usage

List defined jobs:

```sh
cargo ci --list
```

Run a single job:

```sh
cargo ci build
```

Run all jobs sequentially (alphabetical order):

```sh
cargo ci --all-jobs
```

Dry-run (show what would execute without running commands):

```sh
cargo ci build --dry-run
```

Keep going on failures (continue other commands / packages):

```sh
cargo ci build --keep-going
```

Run commands sequentially across packages (instead of parallel):

```sh
cargo ci build --sequential
```

## Workspace Behavior

In a workspace with multiple member crates, each command in a job is executed once per member crate (current working directory set to that crate's directory). In a single-crate project, commands run only once in the root.

## Custom Display Names

By default, cargo-ci displays the full command line in status updates. For long or complex commands, you can provide a custom display name at the job level or step level:

### Job-level custom name

```toml
[workspace.metadata.ci.jobs.complex_build]
name = "Building with custom flags"
steps = ["cargo build --release --all-features --target x86_64-unknown-linux-gnu"]
```

When running this job, instead of showing the entire command, it will display:
```text
[complex_build] (1/1) Building with custom flags
```

### Step-level custom name

For finer control, you can specify a custom name for individual steps using the extended step format:

```toml
[workspace.metadata.ci.jobs.multi_step]
steps = [
  { command = "cargo build --release", name = "Release Build" },
  { command = "cargo test --all-features", name = "Full Test Suite" },
  "cargo clippy"  # Simple format still supported
]
```

This makes the output cleaner and more readable, especially for jobs with very long command lines or when using shell scripts with many arguments.

## Modifiers

Modifiers allow you to conditionally execute steps based on project-specific tags. This is useful for running different commands depending on the build environment, feature set, or development phase.

### Defining Project Modifiers

Define modifiers in the `[package.metadata.ci]` or `[workspace.metadata.ci]` section:

```toml
[package.metadata.ci]
modifiers = ["nightly", "experimental"]
```

### Using Modifiers in Steps

Steps can include a `modifiers` field with a boolean expression that determines whether the step should execute:

```toml
[workspace.metadata.ci.jobs.conditional_build]
steps = [
  "cargo build",  # Always runs
  { command = "cargo build --features experimental", modifiers = "experimental" },
  { command = "cargo +nightly build", modifiers = "nightly" },
  { command = "cargo test --release", modifiers = "nightly & experimental" }
]
```

### Modifier Expression Syntax

Modifier expressions support boolean logic:

- **Identifier**: `nightly`, `experimental`, etc. - evaluates to `true` if the modifier is defined
- **Logical AND**: `&` - both operands must be true
- **Logical OR**: `|` - at least one operand must be true
- **Logical NOT**: `!` - negates the following expression
- **Grouping**: `(` and `)` - controls evaluation order

### Examples

```toml
# Run only if "nightly" modifier is defined
{ command = "cargo +nightly build", modifiers = "nightly" }

# Run if both "nightly" AND "experimental" are defined
{ command = "cargo test --all-features", modifiers = "nightly & experimental" }

# Run if either "nightly" OR "stable" is defined
{ command = "cargo build", modifiers = "nightly | stable" }

# Run if "nightly" is NOT defined
{ command = "cargo build --no-unstable", modifiers = "!nightly" }

# Complex expression with grouping
{ command = "cargo test", modifiers = "(nightly | beta) & !windows" }
```

### Operator Precedence

Operators are evaluated in the following order (highest to lowest precedence):
1. `!` (NOT)
2. `&` (AND)
3. `|` (OR)

Use parentheses to override the default precedence:
- `a | b & c` is evaluated as `a | (b & c)`
- `(a | b) & c` forces the OR to be evaluated first

### Behavior

- Steps without a `modifiers` field always execute
- Steps with a `modifiers` field are skipped if the expression evaluates to false
- When a step is skipped, it's indicated in the output: `(skipped due to modifiers)`
- Invalid modifier expressions cause the job to fail with an error message

### Use Cases

Modifiers are useful for:

- **Development phases**: Different commands for alpha, beta, release candidates, and production
- **Platform-specific builds**: Run commands only on specific operating systems
- **Feature flags**: Test experimental features separately from stable ones
- **Toolchain versions**: Use nightly-only features when the nightly modifier is set
- **CI environments**: Different behavior for local development vs. CI servers

## Environment Variables

When executing steps, cargo-ci sets the following environment variables:

- `CI_JOB`: Name of the current job being executed
- `CI_PACKAGE_NAME`: Name of the package in which the step is running

These can be used in your scripts to customize behavior based on the job context.

<!-- cargo-rdme end -->
