pub static FILES: [(&'static str, &'static [u8]); 2] = [
    ("/bin/shell", b"#!/bin/tri\n# Shell TRI v0.1 - echo 'Booted!'"),
    ("/etc/tri-shellrc", b"export TRI_RATIO=177\nset prompt='tri-root@kernel:~#'"),
];

pub fn read_file(path: &str) -> Option<&'static [u8]> {
    for &(name, content) in FILES.iter() {
        if name == path { return Some(content); }
    }
    None
}

pub fn list_files() -> &'static [&'static str] {
    &["/bin/shell", "/etc/tri-shellrc"]
}
