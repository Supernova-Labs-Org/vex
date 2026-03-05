# Contributing to vex

Thanks for your interest in contributing! Here's how to get started.

## Setup

1. Clone the repository:
```bash
git clone https://github.com/nishujangra/vex.git
cd vex
```

2. Build the project:
```bash
cargo build
```

3. Run tests:
```bash
cargo test
```

## Making Changes

1. Create a new branch for your changes:
```bash
git checkout -b fix/issue-name
```

2. Make your changes and ensure:
   - Code compiles: `cargo build`
   - Tests pass: `cargo test`
   - Code is clean: `cargo clippy -- -D warnings`
   - Formatting is correct: `cargo fmt`

3. Commit with clear messages:
```bash
git commit -m "fix: brief description of the fix"
```

## Documentation

If your changes affect CLI behavior or metrics:
- Update `docs/CLI_REFERENCE.md` for new options
- Update `docs/METRICS.md` for metric changes
- Update `README.md` for feature changes

## Testing

Before submitting:
```bash
cargo test
cargo build --release
cargo clippy -- -D warnings
```

## Pull Requests

1. Push to your fork and create a PR to `master`
2. Link related issues: `Closes #123`
3. Describe what changed and why
4. Tests must pass

## Code Style

- Follow Rust conventions (enforced by `cargo fmt`)
- Keep functions focused and readable
- Add doc comments for public APIs
- No unnecessary dependencies

## Questions?

Open an issue or discussion. We're happy to help!
