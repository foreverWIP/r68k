use std::fs;
// build.rs
use std::process::Command;
use std::{env, path::Path};

fn main() {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&cargo_manifest_dir);
    let manifest_path_parent = manifest_path.parent().unwrap();

    let musashi_dir = manifest_path_parent.join(Path::new("Musashi"));

    let m68kmake_file = musashi_dir.join(Path::new("m68kmake.c"));
    println!("Building m68kmake...");
    let m68kmake_file_str = m68kmake_file.to_str().unwrap();
    let m68kmake_compiler = cc::Build::new().file(m68kmake_file_str).get_compiler();
    let cc_name = m68kmake_compiler.path();
    if m68kmake_compiler.is_like_msvc() {
        match Command::new(cc_name)
            .arg(&format!("{}", m68kmake_file_str))
            .current_dir(&musashi_dir)
            .envs(
                m68kmake_compiler
                    .env()
                    .iter()
                    .map(|it| (it.0.clone(), it.1.clone())),
            )
            .status()
        {
            Ok(_) => {}
            Err(e) => panic!("Compiling m68kmake failed!: {}", e.to_string()),
        };
        match Command::new(musashi_dir.join(Path::new("m68kmake.exe")))
            .current_dir(&musashi_dir)
            .status()
        {
            Ok(_) => {}
            Err(e) => panic!("Running m68kmake failed!: {}", e.to_string()),
        };
        cc::Build::new()
            .file(musashi_dir.join(Path::new("m68kcpu.c")))
            .file(musashi_dir.join(Path::new("m68kops.c")))
            .file(
                musashi_dir
                    .join(Path::new("softfloat"))
                    .join(Path::new("softfloat.c")),
            )
            .opt_level(2)
            .compile("musashi");
        let _ = fs::remove_file(musashi_dir.join(Path::new("m68kmake.exe")));
        let _ = fs::remove_file(musashi_dir.join(Path::new("m68kmake.obj")));
    } else {
        match Command::new(cc_name)
            .args(&["-o", "m68kmake"])
            .arg(&format!("{}", m68kmake_file_str))
            .current_dir(&musashi_dir)
            .status()
        {
            Ok(_) => {}
            Err(e) => panic!("Compiling m68kmake failed!: {}", e.to_string()),
        }
        match Command::new(musashi_dir.join(Path::new("m68kmake")))
            .current_dir(&musashi_dir)
            .status()
        {
            Ok(_) => {}
            Err(e) => panic!("Running m68kmake failed!: {}", e.to_string()),
        };
        cc::Build::new()
            .file(musashi_dir.join(Path::new("m68kcpu.c")))
            .file(musashi_dir.join(Path::new("m68kops.c")))
            .file(
                musashi_dir
                    .join(Path::new("softfloat"))
                    .join(Path::new("softfloat.c")),
            )
            .opt_level(2)
            .compile("musashi");
        let _ = fs::remove_file(musashi_dir.join(Path::new("m68kmake")));
        let _ = fs::remove_file(musashi_dir.join(Path::new("m68kmake.o")));
    }
    println!("cargo:rerun-if-changed={}", musashi_dir.to_str().unwrap());
}
