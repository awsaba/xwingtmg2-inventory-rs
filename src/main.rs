use std::path::Path;

mod xwingdata2;

fn main() {
    println!("Hello, world!");

    match xwingdata2::load_from_manifest(Path::new("xwing-data2")) {
        Ok(d) => println!("{:?}", d),
        Err(e) => println!("{:?}", e),
    }
}
