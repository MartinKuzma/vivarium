#[allow(unused_imports)]
use crate::core::persistence::loader;

#[test]
fn test_load_project_from_file() {
    let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("predator-prey")
        .join("world.yaml");

    let project = loader::load_project_from_file(manifest_path.to_str().unwrap())
        .expect("expected example project manifest to load");

    assert_eq!(project.manifest.schema_version, "v1");
    assert_eq!(project.manifest.name, "ecosystem_v2");
    assert!(project.manifest.script_library.contains_key("prey_script"));

    let script = project
        .manifest
        .script_library
        .get("prey_script")
        .expect("script should be present in project manifest");

    assert_eq!(script.id, "prey_script");
    assert_eq!(script.kind, "lua");

    let expected_project_root = manifest_path
        .parent()
        .expect("manifest path should have a parent")
        .to_path_buf();
    assert_eq!(project.project_root, expected_project_root);
}
