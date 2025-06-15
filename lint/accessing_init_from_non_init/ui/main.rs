static FOO: u32 = 5;

#[unsafe(link_section = ".init.rodata")]
static BAR: u64 = 15;

#[unsafe(link_section = ".init.data")]
static OWO: usize = 100;

fn main() {
    let x = FOO + 10 * FOO.trailing_zeros();

    let y = BAR + 10 * BAR.trailing_zeros() as u64;

    let z = OWO + 10 * OWO.trailing_zeros() as usize;
}
