### What it does

Checks for calls to functions marked as `#[init]` from functions not marked as `#[init]`.

### Why is this bad?

Functions marked as `#[init]` should only be called from other `#[init]` functions,
since they might be unmapped from the kernel once the init phase completes.

### Known problems

It does not detect indirect function calls through functions pointers or closures,
since the information if they are marked as `#[init]` or not is lost on conversion.

### Example

```rust
#[init]
fn foo() {}

fn bar() {
    foo();
}
```

Use instead:

```rust
#[init]
fn foo() {}

#[init]
fn bar() {
    foo();
}
```