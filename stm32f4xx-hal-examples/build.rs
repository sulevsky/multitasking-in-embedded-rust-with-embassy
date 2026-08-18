fn main() {
    println!("cargo::rustc-link-arg=--nmagic");
    println!("cargo::rustc-link-arg=-Tlink.x");
    println!("cargo::rustc-link-arg=-Tdefmt.x");

    // add stm32-target dir for linker search to find memory.x
    println!("cargo::rustc-link-search=stm32-hal");
}
