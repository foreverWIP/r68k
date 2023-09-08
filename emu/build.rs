extern crate git2;
use git2::{Diff, Repository};
use std::fs;
use std::process::Command;
use std::{env, path::Path};

fn get_build_profile_name() -> String {
    // The profile name is always the 3rd last part of the path (with 1 based indexing).
    // e.g. /code/core/target/cli/build/my-build-info-9f91ba6f99d7a061/out
    std::env::var("OUT_DIR")
        .unwrap()
        .split(std::path::MAIN_SEPARATOR)
        .nth_back(3)
        .unwrap_or_else(|| "unknown")
        .to_string()
}

const DIFF: &str = r#"diff --git a/m68kconf.h b/m68kconf.h
index 8844952..4862f40 100644
--- a/m68kconf.h
+++ b/m68kconf.h
@@ -64,11 +64,11 @@
 /* ======================================================================== */
 
 /* Turn ON if you want to use the following M68K variants */
-#define M68K_EMULATE_010            OPT_ON
-#define M68K_EMULATE_EC020          OPT_ON
-#define M68K_EMULATE_020            OPT_ON
-#define M68K_EMULATE_030            OPT_ON
-#define M68K_EMULATE_040            OPT_ON
+#define M68K_EMULATE_010            OPT_OFF
+#define M68K_EMULATE_EC020          OPT_OFF
+#define M68K_EMULATE_020            OPT_OFF
+#define M68K_EMULATE_030            OPT_OFF
+#define M68K_EMULATE_040            OPT_OFF
 
 
 /* If ON, the CPU will call m68k_read_immediate_xx() for immediate addressing
@@ -82,15 +82,15 @@
  * To simulate real 68k behavior, m68k_write_32_pd() must first write the high
  * word to [address+2], and then write the low word to [address].
  */
-#define M68K_SIMULATE_PD_WRITES     OPT_OFF
+#define M68K_SIMULATE_PD_WRITES     OPT_ON
 
 /* If ON, CPU will call the interrupt acknowledge callback when it services an
  * interrupt.
  * If off, all interrupts will be autovectored and all interrupt requests will
  * auto-clear when the interrupt is serviced.
  */
-#define M68K_EMULATE_INT_ACK        OPT_OFF
-#define M68K_INT_ACK_CALLBACK(A)    your_int_ack_handler_function(A)
+#define M68K_EMULATE_INT_ACK        OPT_ON
+#define M68K_INT_ACK_CALLBACK(A)    cpu_irq_ack(A)
 
 
 /* If ON, CPU will call the breakpoint acknowledge callback when it encounters
@@ -147,8 +147,8 @@
  * want to properly emulate the m68010 or higher. (moves uses function codes
  * to read/write data from different address spaces)
  */
-#define M68K_EMULATE_FC             OPT_OFF
-#define M68K_SET_FC_CALLBACK(A)     your_set_fc_handler_function(A)
+#define M68K_EMULATE_FC             OPT_SPECIFY_HANDLER
+#define M68K_SET_FC_CALLBACK(A)     m68k_set_fc(A)
 
 /* If ON, CPU will call the pc changed callback when it changes the PC by a
  * large value.  This allows host programs to be nicer when it comes to
@@ -161,19 +161,19 @@
 /* If ON, CPU will call the instruction hook callback before every
  * instruction.
  */
-#define M68K_INSTRUCTION_HOOK       OPT_OFF
-#define M68K_INSTRUCTION_CALLBACK(pc) your_instruction_hook_function(pc)
+#define M68K_INSTRUCTION_HOOK       OPT_SPECIFY_HANDLER
+#define M68K_INSTRUCTION_CALLBACK(pc) cpu_instr_callback(pc)
 
 
 /* If ON, the CPU will emulate the 4-byte prefetch queue of a real 68000 */
-#define M68K_EMULATE_PREFETCH       OPT_OFF
+#define M68K_EMULATE_PREFETCH       OPT_ON
 
 
 /* If ON, the CPU will generate address error exceptions if it tries to
  * access a word or longword at an odd address.
  * NOTE: This is only emulated properly for 68000 mode.
  */
-#define M68K_EMULATE_ADDRESS_ERROR  OPT_OFF
+#define M68K_EMULATE_ADDRESS_ERROR  OPT_ON
 
 
 /* Turn ON to enable logging of illegal instruction calls.
@@ -186,7 +186,7 @@
 
 /* Emulate PMMU : if you enable this, there will be a test to see if the current chip has some enabled pmmu added to every memory access,
  * so enable this only if it's useful */
-#define M68K_EMULATE_PMMU   OPT_ON
+#define M68K_EMULATE_PMMU   OPT_OFF
 
 /* ----------------------------- COMPATIBILITY ---------------------------- */
 
"#;

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
    } else {
        match fs::copy(m68kconf_file.clone().with_extension("h.bak"), m68kconf_file) {
            Ok(_) => {}
            Err(_) => panic!("unable to back up conf file!"),
        }
    }
    let target_repo = Repository::open(musashi_dir.to_str().unwrap()).unwrap();
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
    }
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
