use crate::core::snapshot::WorldSnapshot;
use crate::core::world::World;
use crate::core::world_config::WorldCfg;

#[test]
fn test_load_from_file() {
    let world_cfg = WorldCfg::from_json_file("src/core/tests/test_world_config.json").unwrap();
    assert_eq!(world_cfg.name, "file_test_world");
    assert_eq!(world_cfg.entities.len(), 2);
    assert_eq!(world_cfg.entities[0].id, "entity_a");
    assert_eq!(world_cfg.entities[0].script_id, "script_a");
    assert_eq!(world_cfg.entities[1].id, "entity_b");
    assert_eq!(world_cfg.entities[1].script_id, "script_b");

    // build world
    _ = World::new(&world_cfg).unwrap();
}

#[test]
fn test_load_from_yaml_file() {
    let world_cfg = WorldCfg::from_yaml_file("src/core/tests/test_world_config.yaml").unwrap();
    assert_eq!(world_cfg.name, "yaml_test_world");
    assert_eq!(world_cfg.entities.len(), 2);
    assert_eq!(world_cfg.entities[0].id, "entity_x");
    assert_eq!(world_cfg.entities[0].script_id, "script_x");
    assert_eq!(world_cfg.entities[1].id, "entity_y");
    assert_eq!(world_cfg.entities[1].script_id, "script_y");

    // build world
    _ = World::new(&world_cfg).unwrap();
}

#[test]
fn test_snapshosts() {
    let world_cfg = WorldCfg::from_yaml_file("src/core/tests/test_world_config.yaml").unwrap();
    let world = World::new(&world_cfg).unwrap();

    let snapshot = world.create_snapshot().unwrap();

    assert_eq!(snapshot.configuration.name, "yaml_test_world");
    assert_eq!(snapshot.configuration.entities.len(), 2);

    // Entities are preserved in snapshot

    snapshot.configuration.entities.iter().for_each(|entity_cfg| {
        let original_entity_cfg = world_cfg
            .entities
            .iter()
            .find(|e| e.id == entity_cfg.id)
            .unwrap();
        assert_eq!(entity_cfg.script_id, original_entity_cfg.script_id);
    });
}
