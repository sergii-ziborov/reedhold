//! Crate dependency firewall. Matches `.weavatrix/architecture.json`.

use std::fs;
use std::path::PathBuf;

const FORBIDDEN: &[(&str, &[&str])] = &[
    (
        "reedhold-core",
        &[
            "reedhold-codec",
            "reedhold-identity",
            "reedhold-recovery",
            "reedhold-event",
            "reedhold-protocol",
            "reedhold-mesh",
            "reedhold-storage",
            "reedhold-chain",
            "reedhold-client",
            "reedhold",
        ],
    ),
    (
        "reedhold-codec",
        &[
            "reedhold-identity",
            "reedhold-protocol",
            "reedhold-mesh",
            "reedhold-client",
            "reedhold",
        ],
    ),
    (
        "reedhold-identity",
        &[
            "reedhold-recovery",
            "reedhold-event",
            "reedhold-protocol",
            "reedhold-mesh",
            "reedhold-client",
            "reedhold",
        ],
    ),
    (
        "reedhold-recovery",
        &[
            "reedhold-event",
            "reedhold-protocol",
            "reedhold-mesh",
            "reedhold-client",
            "reedhold",
        ],
    ),
    (
        "reedhold-event",
        &[
            "reedhold-recovery",
            "reedhold-protocol",
            "reedhold-mesh",
            "reedhold-client",
            "reedhold",
        ],
    ),
    (
        "reedhold-protocol",
        &[
            "reedhold-mesh",
            "reedhold-storage",
            "reedhold-chain",
            "reedhold-client",
            "reedhold",
        ],
    ),
    (
        "reedhold-mesh",
        &[
            "reedhold-codec",
            "reedhold-identity",
            "reedhold-protocol",
            "reedhold-storage",
            "reedhold-chain",
            "reedhold-client",
            "reedhold",
        ],
    ),
    (
        "reedhold-storage",
        &[
            "reedhold-codec",
            "reedhold-identity",
            "reedhold-protocol",
            "reedhold-mesh",
            "reedhold-chain",
            "reedhold-client",
            "reedhold",
        ],
    ),
    (
        "reedhold-chain",
        &[
            "reedhold-codec",
            "reedhold-identity",
            "reedhold-protocol",
            "reedhold-mesh",
            "reedhold-storage",
            "reedhold-client",
            "reedhold",
        ],
    ),
];

#[test]
fn crate_manifests_respect_the_layering() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (crate_name, forbidden) in FORBIDDEN {
        assert_no_dep(&root, crate_name, forbidden);
    }
}

fn assert_no_dep(root: &std::path::Path, crate_name: &str, forbidden: &[&str]) {
    let manifest = fs::read_to_string(root.join(format!("crates/{crate_name}/Cargo.toml")))
        .unwrap_or_else(|_| panic!("read {crate_name} manifest"));
    for name in forbidden {
        assert!(
            !declares_dependency(&manifest, name),
            "{crate_name} must not depend on {name}"
        );
    }
}

fn declares_dependency(manifest: &str, crate_name: &str) -> bool {
    let Some(deps) = manifest.split("[dependencies]").nth(1) else {
        return false;
    };
    let section = deps.split('[').next().unwrap_or(deps);
    section.lines().any(|line| {
        let trimmed = line.trim();
        trimmed == crate_name
            || trimmed.starts_with(&format!("{crate_name}."))
            || trimmed.starts_with(&format!("{crate_name} "))
            || trimmed.starts_with(&format!("{crate_name}="))
    })
}
