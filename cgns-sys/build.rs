use std::env;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

const TAG: &str = "v4.4.0";

macro_rules! log {
    ($fmt:expr) => (println!(concat!("cgns-sys/build.rs:{}: ", $fmt), line!()));
    ($fmt:expr, $($arg:tt)*) => (println!(concat!("cgns-sys/build.rs:{}: ", $fmt),
    line!(), $($arg)*));
}

#[derive(Debug)]
struct ParseCallbacks();

impl bindgen::callbacks::ParseCallbacks for ParseCallbacks {
    fn add_derives(&self, info: &bindgen::callbacks::DeriveInfo<'_>) -> Vec<String> {
        if info.name == "ElementType_t" {
            vec![
                "num_derive::FromPrimitive".into(),
                "num_derive::ToPrimitive".into(),
            ]
        } else {
            vec![]
        }
    }
}

fn main() {
    let static_link = !cfg!(feature = "dynamic");

    let path_out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let path_cgns = path_out_dir.join("CGNS");
    let path_cgns_src = path_cgns.join("src");

    if !path_cgns.join(".git").exists() {
        run("git", true, |command| {
            command
                .arg("clone")
                .arg(format!("--branch={}", TAG))
                .arg("--recursive")
                .arg("https://github.com/CGNS/CGNS.git")
                .arg(&path_cgns);
        });
    } else {
        let git_reset_hard_cmd = |cmd: &mut Command| {
            cmd.current_dir(&path_cgns).args(["reset", "--hard", TAG]);
        };
        if !run("git", false, git_reset_hard_cmd) {
            run("git", true, |cmd| {
                cmd.current_dir(&path_cgns).arg("fetch");
            });
            run("git", true, git_reset_hard_cmd);
        }
    }

    for patch in fs::read_dir("./patches").unwrap() {
        let path = patch.unwrap().path().canonicalize().expect("Can't get canonical path to patch");
        log!("Applying patch: {:?}", path);
        run("git", true, |cmd| {
            cmd.current_dir(&path_cgns).arg("apply").arg(&path);
        });
    }

    let mut path_cgns_build = path_out_dir; // dummy value
    if static_link {
        path_cgns_build = cmake::Config::new(path_cgns).pic(true).build();
        println!(
            "cargo:rustc-link-search=native={}",
            path_cgns_build.join("lib").display()
        );
        println!("cargo:rustc-link-lib=static=cgns");
        println!("cargo:rustc-link-lib=hdf5");
    } else {
        println!("cargo:rustc-link-lib=cgns");
    }

    let bindings = bindgen::Builder::default()
        .clang_arg(format!("-F{}", path_cgns_src.display()))
        .clang_arg(format!("-F{}", path_cgns_build.join("include").display()))
        .default_enum_style(bindgen::EnumVariation::Rust {
            non_exhaustive: true,
        })
        .parse_callbacks(Box::new(ParseCallbacks()))
        .size_t_is_usize(true)
        .header(path_cgns_src.join("cgnslib.h").to_str().unwrap())
        .generate()
        .expect("Could not generate cgnslib bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("cgnslib.rs"))
        .expect("Could not write cgnslib.rs");
}

fn run<F>(name: &str, assert_success: bool, mut configure: F) -> bool
where
    F: FnMut(&mut Command),
{
    let mut command = Command::new(name);
    configure(&mut command);
    log!("Executing {:?}", command);
    let success = match command.status() {
        Err(_) => panic!("Could not execute {:?}", command),
        Ok(s) => s.success(),
    };
    if assert_success && !success {
        panic!("Command was not successful: {:?}", command);
    }
    log!("Command {:?} finished with success: {}", command, success);
    success
}
