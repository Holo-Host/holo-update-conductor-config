{ sources ? import ./nix/sources.nix { }
, pkgs ? import sources.holo-nixpkgs { }
, nixpkgs ? import sources.nixpkgs { }
}:

with pkgs;

nixpkgs.rustPlatform.buildRustPackage {
  name = "holo-update-conductor-config";
  src = gitignoreSource ./.;
  cargoSha256 = "1zkbs9nzh4gvadal0jjp7d67in6d9756q1djvinpjy7k85rydhl4";

  meta.platforms = lib.platforms.linux;
}
