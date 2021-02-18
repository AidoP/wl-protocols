# Wl Protocols

[Wayland Protocols](https://gitlab.freedesktop.org/wayland/wayland-protocols) but in TOML. Included in this repository is a tool for converting XML Wayland protocol specifications to TOML as well as the protocols themselves under `protocol/`. Built for [wl](https://github.com/AidoP/wl), but applicable for other Wayland libraries which choose to abandon XML.

The protocol files fall under their original licences.

# Usage

## Dependencies
- Rust and Cargo
- [wayland-protocols](https://gitlab.freedesktop.org/wayland/wayland-protocols)

## Default Protocols

```sh
sh convert
```

## Install the tool

```sh
cargo install --path .
```

## Ensure Integrity

Will test that the in-memory representation for the xml and toml versions are identical. This test is not perfect but should catch most issues.

```sh
# Convert first
sh convert
cargo test
```


## Manual Usage

```sh
# A single file
wl-protocol /usr/share/wayland/wayland.xml -o protocol/wayland.xml
# All XML files in a subdirectory (preserving structure)
wl-protocol /usr/shar/wayland-protocols/ -o protocol/
# To stdout
wl-protocol file.xml
```