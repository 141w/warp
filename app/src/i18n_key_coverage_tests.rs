//! Guards against a translation key being referenced from code but absent
//! from the locale files, which rust-i18n renders as the raw key in the UI.

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .expect("app crate has a parent")
        .to_path_buf()
}

fn collect_rs_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let name = path.file_name().unwrap_or_default().to_string_lossy();
            if name == "target" || name == ".git" {
                continue;
            }
            collect_rs_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

fn keys_in(src: &str) -> Vec<String> {
    let mut keys = Vec::new();
    for marker in ["t!(\"", "warp_i18n::tr(\""] {
        let mut rest = src;
        while let Some(i) = rest.find(marker) {
            rest = &rest[i + marker.len()..];
            if let Some(end) = rest.find('\"') {
                let key = &rest[..end];
                if key
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == ':')
                    && key.contains('.')
                {
                    keys.push(key.to_string());
                }
            }
        }
    }
    keys
}

fn flatten(value: &serde_yaml::Value, prefix: &str, out: &mut Vec<String>) {
    match value {
        serde_yaml::Value::Mapping(map) => {
            for (k, v) in map {
                let Some(k) = k.as_str() else { continue };
                let path = if prefix.is_empty() {
                    k.to_string()
                } else {
                    format!("{prefix}.{k}")
                };
                flatten(v, &path, out);
            }
        }
        _ => {
            if !prefix.is_empty() {
                out.push(prefix.to_string());
            }
        }
    }
}

#[test]
fn every_referenced_key_exists_in_the_locale_file() {
    let root = repo_root();
    let en_text = fs::read_to_string(root.join("crates/warp_i18n/locales/en.yml"))
        .expect("en.yml is readable");
    let en: serde_yaml::Value = serde_yaml::from_str(&en_text).expect("en.yml parses");

    let mut present = Vec::new();
    flatten(&en, "", &mut present);
    let present: std::collections::HashSet<String> = present.into_iter().collect();

    let mut files = Vec::new();
    collect_rs_files(&root.join("app/src"), &mut files);
    collect_rs_files(&root.join("crates"), &mut files);

    // Keys that deliberately do not exist, to exercise the fallback path.
    const INTENTIONALLY_ABSENT: &[&str] = &["no.such.key.at.all", "a.b.c"];

    let mut missing = Vec::new();
    for file in &files {
        let Ok(src) = fs::read_to_string(file) else {
            continue;
        };
        for key in keys_in(&src) {
            if INTENTIONALLY_ABSENT.contains(&key.as_str()) {
                continue;
            }
            if !present.contains(&key) {
                missing.push(format!(
                    "{} -> {}",
                    file.strip_prefix(&root).unwrap_or(file).display(),
                    key
                ));
            }
        }
    }

    missing.sort();
    missing.dedup();
    assert!(
        missing.is_empty(),
        "keys referenced from code but missing from en.yml ({}): {:?}",
        missing.len(),
        missing
    );
}
