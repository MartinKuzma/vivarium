#[allow(unused_imports)]
use crate::core::persistence::loader;

#[test]
fn test_load_project_from_manifest_file_with_latest_snapshot() {
    let manifest_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("examples")
        .join("predator-prey")
        .join("world.yaml");

    let project = loader::load_project_from_manifest_file(
        manifest_path.to_str().unwrap(),
        loader::SnapshotSelection::Latest,
    )
    .expect("expected example project manifest and latest snapshot to load");

    assert_eq!(project.manifest.schema_version, "v1");
    assert_eq!(project.manifest.name, "ecosystem_v2");
    assert!(project.manifest.script_library.contains_key("prey_script"));

    assert_eq!(project.world_cfg.name, "ecosystem_v2");
    assert_eq!(project.world_cfg.simulation_time, 0);
    assert_eq!(project.world_cfg.pending_messages.len(), 0);
    assert_eq!(project.world_cfg.entities.len(), 1);

    let entity = &project.world_cfg.entities[0];
    assert_eq!(entity.id, "prey");
    assert_eq!(entity.script_id, "prey_script");

    let script = project
        .world_cfg
        .script_library
        .get("prey_script")
        .expect("script should be present in world config");

    assert_eq!(script.id, "prey_script");
    assert_eq!(script.kind, "lua");
    assert!(script.script.contains("function update(current_time, msgs)"));
}
