let pkgs = import <nixpkgs> { };
in pkgs.rustPlatform.buildRustPackage rec {
  pname = "k4status";
  version = "1.2.0";
  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "spaceapi-0.9.0" = "sha256-SApu1fkGqfMTjg9VoLj5qolzVTiv/lZ2fIYVHCqWDb0=";
    };
  };
  src = pkgs.lib.cleanSource ./.;
  postInstall = ''
    mkdir -p $out/conf
    cp template.json $out/conf/template.json
  '';
}
