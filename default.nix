{ sources ? import ./nix/sources.nix { }
, pkgs ? import sources.holo-nixpkgs { }
, nixpkgs ? import sources.nixpkgs { }
}:

with pkgs;

nixpkgs.rustPlatform.buildRustPackage {
  name = "holo-update-conductor-config";
  src = gitignoreSource ./.;
  cargoSha256 = "1vi51xqdy5qmk6v59ck76ksi29a7s4wilivqbbg107i09s3r8dik";

  meta.platforms = lib.platforms.linux;
}
