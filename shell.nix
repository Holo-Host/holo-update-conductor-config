{ pkgs ? import <nixpkgs> { } }:
with pkgs;
mkShell { buildInputs = [ openssl pkg-config ]; }
