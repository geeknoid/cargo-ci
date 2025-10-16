# TODO

- Define output variables for all jobs as they run
- Support per-job variables
- Support per-step variables
- Option to turn coloring off
- Should have a deterministic job order, right now it's based on iterating a hashmap.
- Add an 'install' field in a step definition indicating the tool the step is using
- Support splitting the config file into multiple files
- Support reuse of steps across jobs
- Support job sharing via some external reference mechanism
- Fix variable precedence: command-line > package metadata > environment
- Can't have jobs called "run", "install", or "list-jobs" because those clash with subcommands
- Use the `target` directory from Cargo-metadata instead of hardcoding to target
- Make the `install` command output the same logs on failures as the run command
