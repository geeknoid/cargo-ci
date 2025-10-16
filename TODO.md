# TODO

## Features

- Support the ability for steps to execute Rust scripts
- Define output variables for all steps as they run so that subsequent steps can factor those variables in their behavior
- Define variables:
  - CARGO_CI_JOB_NAME = <current job name>
  - CARGO_CI_JOB_ID = <current job id>
  - CARGO_CI_STEP_NAME = <current step name>
  - CARGO_CI_STEP_ID = <current step name>
  - CARGO_CI_GIT_BRANCH = <current git branch name>
  - CARGO_CI_PACKAGE_NAME = <current package name>

## Use in CI/CD

- Add a `uses` field in a step definition indicating the tool the step is using
- Add an --install option to the run command to install missing tools automatically, based on the `uses` fields for each step

## Flexibility

- Support splitting the config file into multiple files
- Support reuse of steps across jobs
- Support job sharing via some external reference mechanism

## Performance

- Use 'cargo install --list' to figure out what's available, before trying to install anything
