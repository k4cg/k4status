let pkgs = import <nixpkgs> { };
in pkgs.rustPlatform.buildRustPackage rec {
  pname = "k4status";
  version = "1.5.1";
  cargoLock = {
    lockFile = ./Cargo.lock;
  };
  src = pkgs.lib.cleanSource ./.;
  postInstall = ''
    mkdir -p $out/conf
    cp template.json $out/conf/template.json
    cp -r assets/ $out/conf/
  '';
}
