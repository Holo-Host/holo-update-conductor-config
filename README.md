# holo-update-conductor-config

![Rust](https://github.com/holo-host/holo-update-conductor-config/workflows/Rust/badge.svg)
![Dependabot](https://badgen.net/dependabot/Holo-Host/holo-update-conductor-config?icon=dependabot)

A CLI program to update `conductor-config.toml` of `holochain-conductor` with settings set in HPOS.

Detailed description can be found [in the RFC](https://github.com/Holo-Host/rfcs/blob/master/conductor-config/README.md).

## Usage

```sh
holo-update-conductor-config /path/to/conductor-config.toml < /path/to/config-from-hpos.toml
```

`/path/to/conductor-config.toml` gets read and then written back.
