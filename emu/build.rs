use std::fs;
use std::process::Command;
use std::{env, path::Path};

const NEW_CONF: &str = include_str!("m68kconf.h");

fn main() {
    let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let manifest_path = Path::new(&cargo_manifest_dir);
    let manifest_path_parent = manifest_path.parent().unwrap();

    let musashi_dir = manifest_path_parent.join(Path::new("Musashi"));

    let m68kmake_file = musashi_dir.join(Path::new("m68kmake.c"));
    let m68kconf_file = musashi_dir.join(Path::new("m68kconf.h"));
    if !m68kconf_file.with_extension("h.bak").exists() {
        match fs::copy(m68kconf_file.clone(), m68kconf_file.with_extension("h.bak")) {
            Ok(_) => {}
            Err(_) => panic!("unable to back up conf file!"),
        }
    } else if !m68kconf_file.exists() {
        match fs::write(m68kconf_file, NEW_CONF) {
            Ok(_) => {}
            Err(_) => panic!("unable to replace conf file!"),
        }
    }
    /*let target_repo = Repository::open(musashi_dir.to_str().unwrap()).unwrap();
    let diff_from_patch = Diff::from_buffer(DIFF.as_bytes()).unwrap();
    let mut apply_opts = git2::ApplyOptions::new();
    apply_opts.check(false);
    match target_repo.apply(
        &diff_from_patch,
        git2::ApplyLocation::WorkDir,
        Some(&mut apply_opts),
    ) {
        Ok(_) => {}
        Err(err) => panic!("applying diff failed! {}", err.message()),
    }*/
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
