{ sources ? import ./nix/sources.nix { }
, pkgs ? import sources.holo-nixpkgs { }
, nixpkgs ? import sources.nixpkgs { }
}:

with pkgs;

nixpkgs.rustPlatform.buildRustPackage {
  name = "holo-update-conductor-config";
  src = gitignoreSource ./.;
  cargoSha256 = "0kwa3dmnqcljbkksj1zajnkhilrx3ps8rya7qyisv5zs540ipkc2";

  meta.platforms = lib.platforms.linux;
}
