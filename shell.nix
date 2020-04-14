{ sources ? import ./nix/sources.nix { }
, pkgs ? import sources.holo-nixpkgs { }
, niv ? (import sources.niv { }).niv
}:
pkgs.mkShell { buildInputs = [ niv ]; }
