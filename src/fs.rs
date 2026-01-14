pub struct File {
    pub name: &'static str,
    pub content: &'static str,
}

pub const FILES: &[File] = &[
    File { name: "readme.txt", content: "Tm_Os v0.11\nEmbedded RamFS is active!" },
    File { name: "hello.rs", content: "fn main() {\n    println!(\"Hello from RamFS!\");\n}" },
];

pub fn get_file(name: &str) -> Option<&'static File> {
    FILES.iter().find(|&f| f.name == name)
}

pub fn list_files() {
    use crate::print;
    print!("Files: ");
    for f in FILES {
        print!("{}  ", f.name);
    }
    crate::println!("");
}