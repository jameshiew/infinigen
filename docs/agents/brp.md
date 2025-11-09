# Bevy Remote Protocol (BRP) guide

Use this reference whenever you need to run Infinigen without a window and introspect the ECS through Bevy's Remote Protocol.

## 1. Launching a headless + remote session

- The remote API is only available on native targets and requires the `remote` feature.
- Always pair it with `--headless` so the app can run on a terminal-only host.

```bash
# From repo root (dev profile)
just run --features remote -- --headless

# Release profile shortcut
just run-remote -- --headless
```

Tips:
- To keep the terminal free, background the process and capture logs:
  ```bash
  just run --features remote -- --headless \
    > /tmp/infinigen_headless.log 2>&1 & echo $!  # prints PID
  ```
- Remote HTTP listens on `http://127.0.0.1:15702/` by default (see `RemoteHttpPlugin` docs). The endpoint speaks JSON-RPC 2.0 over HTTP POST.
- When finished, stop the app with `kill <PID>` (verify with `ps -p <PID>`).
- First launch may emit warnings about `blocks/types` and `blocks/textures` missing; the engine falls back to default colors/definitions, so you can safely ignore them for BRP work.

## 2. Basic remote calls

All requests are JSON payloads with `jsonrpc`, `id`, `method`, and optional `params`. For quick checks you can use `curl`:

```bash
curl -s -X POST \
  -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":1,"method":"world.list_resources"}' \
  http://127.0.0.1:15702/
```

Useful built-in methods (see `bevy_remote::builtin_methods` for the full list):

| Method | Purpose |
| --- | --- |
| `world.list_components` | Lists every registered component (no params) or components on a specific entity (`{"entity": <id>}`) |
| `world.query` | Runs an ECS query and returns matching entity ids + components |
| `world.get_components` | Fetches specific components for an entity (only if the component implements `Reflect`) |
| `registry.schema` | Dumps the reflect schema (handy for discovering type paths) |

Currently registered/reflectable Infinigen resources:

- `infinigen_plugins::scene::SceneSettings`
- `infinigen_plugins::scene::SceneView`
- `infinigen_plugins::scene::SceneZoom`
- `infinigen_plugins::camera::setup::CameraSettings`
- `infinigen_plugins::world::WorldSettings`

All five live in crates that already depend on Bevy, so adding `Reflect` doesn’t pull Bevy into `infinigen-common`.

## 3. Chunk entities (not available via BRP)

`ChunkPosition`, `LoadedChunk`, and the camera/pending chunk helpers all live in `infinigen-common`, which must stay free of Bevy dependencies. Because of that they no longer implement `Reflect`, so they can’t be registered with `bevy_remote`.

Implications:

- `world.query` requests targeting `infinigen_plugins::scene::LoadedChunk` always return an empty list (the component metadata isn’t in the type registry).
- `world.list_components` only reports reflectable components, so `LoadedChunk` is absent.
- Chunk coordinates are therefore only visible through instrumentation (logs, custom debug overlays, etc.), not the remote HTTP API.

Operational tips until we add a Bevy-facing mirror type:

- Watch the headless log (`tail -f /tmp/infinigen_headless.log | rg -i chunk`) for spawn/despawn messages.
- Enable the debug HUD (`just run` without `--headless`) if you need live chunk counts or bounding boxes.
- If you need structured data, add a temporary resource that mirrors the info you require and registers `Reflect` from within the Bevy crates (keeping `infinigen-common` clean).

## 4. Inspecting runtime resources

With the new registrations you can fetch key resources via `world.get_resources`. Examples:

```bash
# Current horizontal/vertical view distance and zoom
curl -s -X POST -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":2,"method":"world.get_resources","params":{"resource":"infinigen_plugins::scene::SceneView"}}' \
  http://127.0.0.1:15702/

# Camera spawn settings (wx/wy/wz etc.)
curl -s -X POST -H 'content-type: application/json' \
  -d '{"jsonrpc":"2.0","id":3,"method":"world.get_resources","params":{"resource":"infinigen_plugins::camera::setup::CameraSettings"}}' \
  http://127.0.0.1:15702/
```

## 5. Troubleshooting checklist

- **Remote endpoint unreachable**: ensure the app is running with `--features remote` and not compiled for WASM. Confirm port 15702 is free.
- **Immediate exit in headless mode**: verify you are on a build that sets `ExitCondition::DontExit` (current default). If the log still shows `No windows are open, exiting`, rebuild and rerun.
- **HTTP 500 / serialization errors**: the component/resource may not implement `Reflect`. Use `registry.schema` to confirm availability, or fall back to `world.list_components` to spot non-reflectable markers.
- **Need custom queries**: consult `bevy_remote::builtin_methods` for the JSON structures accepted by each verb, then craft the payload accordingly.
