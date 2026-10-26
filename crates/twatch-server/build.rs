//! Stage the SPA into the crate before `rust-embed` reads it.
//!
//! `cargo build --all-features` enables `embed-web` even on a fresh checkout
//! where `web/dist` has not been built yet.  Rather than fail the build, stage
//! a tiny placeholder; a real `npm run build` replaces it.  The staged folder
//! is gitignored, so developer builds stay clean.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

fn copy_dir(source: &Path, target: &Path) {
    let Ok(entries) = fs::read_dir(source) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let destination = target.join(entry.file_name());
        if path.is_dir() {
            let _ = fs::create_dir_all(&destination);
            copy_dir(&path, &destination);
        } else if fs::copy(&path, &destination).is_err() {
            // A partially written asset is not worth failing the Rust build for.
        }
    }
}

fn main() {
    println!("cargo:rerun-if-changed=../../web/dist");
    println!("cargo:rerun-if-changed=../../web/package.json");

    if env::var_os("CARGO_FEATURE_EMBED_WEB").is_none() {
        return;
    }

    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let source = manifest.join("../../web/dist");
    let target = manifest.join("web-dist");

    let _ = fs::remove_dir_all(&target);
    fs::create_dir_all(&target).expect("create web-dist staging directory");

    if source.join("index.html").is_file() {
        copy_dir(&source, &target);
    } else {
        fs::write(
            target.join("index.html"),
            "<!doctype html><html><body style=\"font:15px system-ui;background:#12141a;color:#e4e4e7;padding:3rem\">\
             <h1>twatch web UI is not built</h1>\
             <p>Run <code>cd web &amp;&amp; npm install &amp;&amp; npm run build</code>, then rebuild twatchd.</p>\
             </body></html>",
        )
        .expect("write placeholder index");
    }
}
