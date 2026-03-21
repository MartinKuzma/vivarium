# Project Structure
Each vivarium project is organized in a directory structure:
```text
<project-root>/
├─ world.yaml
├─ scripts/
│  └─ *.lua
└─ snapshots/
	└─ <snapshot-id>/
		├─ snapshot.yaml
		├─ entities.yaml
		└─ messages.yaml
```

| File/Directory | Description |
|----------------|-------------|
| `world.yaml` | Contains the project metadata and references to scripts. |
| `scripts/` | Directory containing scripts that define entity behaviors. |
| `snapshots/` | Directory containing simulation snapshots. Each snapshot is stored in a subdirectory named after its unique ID. This allows returning to a specific point in the simulation. |