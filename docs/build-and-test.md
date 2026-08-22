# Build and test Bevy-GUI

## Local Linux build

Install the native development packages used by the editor windowing/audio stack, then build the binary:

```bash
sudo apt-get update
sudo apt-get install -y \
  pkg-config \
  libasound2-dev \
  libudev-dev \
  libx11-dev \
  libxi-dev \
  libxrandr-dev \
  libxcursor-dev \
  libxinerama-dev \
  libwayland-dev

cargo build --release --bin bevy-gui
./target/release/bevy-gui
```

## Run from a project directory

The editor reads `project.godot-rs.json` from the current working directory when present. Start the editor from the project root:

```bash
cd /path/to/my-project
/path/to/Bevy-GUI/target/release/bevy-gui
```

## GitHub Actions build artifact

The `build` workflow is manual-only. Open **Actions → build → Run workflow**, choose `release` or `debug`, and start it.

The workflow builds the Linux x86_64 editor and uploads an artifact containing:

- `bevy-gui` executable
- `README.md`
- `LICENSE` when present
- a `.tar.gz` bundle

The artifact is retained for 14 days. CI build validation does not run for every commit.
