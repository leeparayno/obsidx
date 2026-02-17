# SQLite VSS Install (macOS)

This documents the local build steps and dependencies for **sqlite‑vss** used by obsidx.

## Prereqs
- Xcode Command Line Tools
- CMake
- Git (with submodules)
- **OpenMP** (required by Faiss)

### Install prereqs (Homebrew)
```bash
brew install cmake libomp
```

## Build sqlite‑vss
```bash
# clone with submodules
mkdir -p /Users/leeparayno/clawd/vendor
cd /Users/leeparayno/clawd/vendor

git clone --recurse-submodules https://github.com/asg017/sqlite-vss.git
cd sqlite-vss

# fetch sqlite amalgamation (if wget missing, use curl)
./vendor/get_sqlite.sh
# or
cd vendor && \
  curl -L https://www.sqlite.org/2022/sqlite-autoconf-3400100.tar.gz -o sqlite.tar.gz && \
  tar -xvzf sqlite.tar.gz && rm sqlite.tar.gz && mv sqlite-autoconf-3400100 sqlite

# build sqlite
cd vendor/sqlite
./configure && make

# build sqlite-vss extensions
cd ../../
make loadable-release
```

## Output artifacts
After `make loadable-release`, the extensions are here:
```
/Users/leeparayno/clawd/vendor/sqlite-vss/dist/release/vector0.dylib
/Users/leeparayno/clawd/vendor/sqlite-vss/dist/release/vss0.dylib
```

## Configure obsidx
Set these env vars so obsidx can load the extensions:
```bash
export OBSIDX_VSS_VECTOR0=/Users/leeparayno/clawd/vendor/sqlite-vss/dist/release/vector0.dylib
export OBSIDX_VSS_VSS0=/Users/leeparayno/clawd/vendor/sqlite-vss/dist/release/vss0.dylib
```

## Test
```bash
obsidx embed-index --vault /path/vault --index /path/.obsidx --vector-backend vss
obsidx embed-search --index /path/.obsidx --query "test" --vector-backend vss --json
```

## Notes
- `sqlite-vss` depends on OpenMP; if you see errors about OpenMP, ensure `libomp` is installed.
- If `wget` is missing, use `curl` as shown above.
- `sqlite-vss` is unmaintained; consider `sqlite-vec` for future migration.
