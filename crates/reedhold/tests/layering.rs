//! Crate dependency firewall. Matches `.weavatrix/architecture.json`.

use std::fs;
use std::path::PathBuf;

const HOSTS: &[&str] = &["reedhold-api", "reedhold-mcp"];
const NETWORK: &[&str] = &[
    "reedhold-mesh",
    "reedhold-storage",
    "reedhold-chain",
    "reedhold-client",
];

#[test]
fn crate_manifests_respect_the_layering() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for crate_name in [
        "reedhold-core",
        "reedhold-codec",
        "reedhold-identity",
        "reedhold-recovery",
        "reedhold-event",
        "reedhold-protocol",
        "reedhold-mesh",
        "reedhold-storage",
        "reedhold-chain",
        "reedhold-client",
        "reedhold-ads",
        "reedhold-store",
        "reedhold-api",
        "reedhold-mcp",
    ] {
        assert_no_dep(&root, crate_name, &["reedhold"]);
    }
    for crate_name in [
        "reedhold-core",
        "reedhold-codec",
        "reedhold-identity",
        "reedhold-recovery",
        "reedhold-event",
        "reedhold-protocol",
        "reedhold-mesh",
        "reedhold-storage",
        "reedhold-chain",
        "reedhold-client",
        "reedhold-ads",
        "reedhold-store",
    ] {
        assert_no_dep(&root, crate_name, HOSTS);
    }
    assert_no_dep(&root, "reedhold-protocol", NETWORK);
    assert_no_dep(&root, "reedhold-api", &["reedhold-client", "reedhold-mcp"]);
    assert_no_dep(&root, "reedhold-ads", NETWORK);
    assert_no_dep(
        &root,
        "reedhold-ads",
        &["reedhold-identity", "reedhold-protocol"],
    );
    assert_no_dep(&root, "reedhold-mesh", &["reedhold-ads"]);
    assert_no_dep(
        &root,
        "reedhold-store",
        &[
            "reedhold-mesh",
            "reedhold-ads",
            "reedhold-identity",
            "reedhold-protocol",
            "reedhold-client",
            "reedhold-chain",
        ],
    );
    assert_no_dep(
        &root,
        "reedhold-mcp",
        &[
            "reedhold-mesh",
            "reedhold-storage",
            "reedhold-chain",
            "reedhold-client",
            "reedhold-identity",
            "reedhold-protocol",
            "reedhold-recovery",
            "reedhold-event",
            "reedhold-codec",
        ],
    );
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
