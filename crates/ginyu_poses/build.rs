use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
};

use proc_macro2::{Literal, TokenStream};
use quote::quote;

fn main() {
    let poses = load_poses();
    let code = generate_code(&poses);

    write_output(&code);

    println!("cargo:rerun-if-changed=poses.txt");
}

fn load_poses() -> Vec<String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("no CARGO_MANIFEST_DIR set");
    let path = Path::new(&manifest_dir).join("poses.txt");
    let file = File::open(&path).expect("Failed to open poses.txt");

    BufReader::new(file)
        .lines()
        .map(|l| l.expect("Failed to read line"))
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

fn generate_code(poses: &[String]) -> TokenStream {
    let static_table = generate_static_table(poses);
    let lookup_fn = generate_lookup_fn(poses);
    let macro_def = generate_macro(poses);

    quote! {
        #static_table
        #lookup_fn
        #macro_def
    }
}

fn generate_static_table(poses: &[String]) -> TokenStream {
    let literals: Vec<_> = poses.iter().map(|a| Literal::string(a)).collect();

    quote! {
        /// Static string table - compile-time known strings.
        pub static STATIC_STRINGS: &[&str] = &[
            #(#literals),*
        ];
    }
}

fn generate_lookup_fn(poses: &[String]) -> TokenStream {
    let arms: Vec<_> = poses
        .iter()
        .enumerate()
        .map(|(index, pose)| {
            let literal = Literal::string(pose);
            quote! { #literal => Some(#index as u32) }
        })
        .collect();

    quote! {
        /// Look up a static pose index by string.
        #[inline]
        pub fn static_pose_index(s: &str) -> Option<u32> {
            match s {
                #(#arms,)*
                _ => None,
            }
        }
    }
}

fn generate_macro(poses: &[String]) -> TokenStream {
    let arms: Vec<_> = poses
        .iter()
        .enumerate()
        .map(|(index, pose)| {
            let literals = Literal::string(pose);
            quote! {
                (#literals) => { $crate::Pose::from_static(#index as u32) }
            }
        })
        .collect();

    quote! {
        /// Create a static Pose at compile time.
        #[macro_export]
        macro_rules! pose {
            #(#arms;)*
            ($s:expr) => { $crate::Pose::from($s) };
        }
    }
}

fn write_output(code: &TokenStream) {
    let out_dir = env::var("OUT_DIR").expect("no OUT_DIR set");
    let dest = Path::new(&out_dir).join("static_poses.rs");
    fs::write(&dest, code.to_string()).expect("Failed to write static_poses.rs");
}
