{ sources ? import ./nix/sources.nix { }
, pkgs ? import sources.holo-nixpkgs { }
, nixpkgs ? import sources.nixpkgs { }
}:

with pkgs;

nixpkgs.rustPlatform.buildRustPackage {
  name = "holo-update-conductor-config";
  src = gitignoreSource ./.;
  cargoSha256 = "10ndvvv3q49pjsffk17zzjlvq51prkskcdywv5hjaqf9h5rmzmnj";

  meta.platforms = lib.platforms.linux;
}
