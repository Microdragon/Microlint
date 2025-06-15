// Plain functions
fn foo() {}

#[unsafe(link_section = ".init.text")]
fn init_foo() {}

// Impl methods
struct A;
impl A {
    fn foo(&self) {}

    #[unsafe(link_section = ".init.text")]
    fn init_foo(&self) {}
}

// Default trait methods
trait AExt {
    fn foo(&self) {}

    #[unsafe(link_section = ".init.text")]
    fn init_foo(&self) {}
}

struct B;
impl AExt for B {}

// Trait method
struct C;
impl AExt for C {
    fn foo(&self) {}

    #[unsafe(link_section = ".init.text")]
    fn init_foo(&self) {}
}

fn main() {
    foo();
    init_foo();
    //~^ calling_init_from_non_init

    let a = A;
    a.foo();
    a.init_foo();
    //~^ calling_init_from_non_init

    let b = B;
    b.foo();
    b.init_foo();
    //~^ calling_init_from_non_init

    let c = C;
    c.foo();
    c.init_foo();
    //~^ calling_init_from_non_init
}
