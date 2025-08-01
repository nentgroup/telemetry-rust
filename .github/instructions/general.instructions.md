---
applyTo: "**.rs"
---

# General Guidelines

## Verification Process

Always go through the following steps to verify your changes:

### 1. Make sure the crate can be built

```bash
cargo check --all-features
```

### 2. Make sure there are no linter errors or warnings
```bash
cargo clippy --all-features --all-targets -- -D warnings
```

### 3. Run tests including doctests
```bash
cargo test --all-features
```

### 4. Verify documentation builds correctly
```bash
cargo doc --all-features --no-deps
```

## Documentation Guidelines

When asked to update crate documentation, prioritize public objects and traits.

Make sure none of those are missing documentation:

```bash
cargo clippy --all-features -- -W missing_docs
```

Also, make sure to update the crate overview in `lib.rs` if any new feature
is added to the crate.

### Public Documentation

Documentation is the main source of information for crate users. 
Keep public objects and traits well documented and add good examples.

### Internal Documentation

Documentation for private objects and traits is optional and should be aimed at crate developers.
It should complement the code base, so keep it short and simple.
It should never be consumed by itself without the code, so you don't have to add examples there
or duplicate any information that could be acquired by reading the code itself.

### Examples

To ensure that examples are valid and compilable, avoid using the `ignore` directive in documentation examples.
All code examples should compile successfully as part of the documentation tests.
