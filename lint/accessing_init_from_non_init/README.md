### What it does

Checks for access to statics marked as `#[init]` from functions not marked as `#[init]`.

### Why is this bad?

Statics marked as `#[init]` should only be accessed from `#[init]` functions,
since they might be unmapped from the kernel once the init phase completes.

### Example

```rust
#[init]
static FOO: u32 = 0;

fn bar() -> u32 {
    FOO + 5
}
```

Use instead:

```rust
#[init]
static FOO: u32 = 0;

#[init]
fn bar() -> u32 {
    FOO + 5
}
```