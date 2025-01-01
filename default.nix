let pkgs = import <nixpkgs> { };
in pkgs.rustPlatform.buildRustPackage rec {
  pname = "k4status";
  version = "1.1.0";
  cargoLock = {
    lockFile = ./Cargo.lock;
    outputHashes = {
      "spaceapi-0.9.0" = "sha256-pfo/JiFetFc/vS2AYQBAukdjxqmdp0Nh/ebwj4C2a10=";
    };
  };
  src = pkgs.lib.cleanSource ./.;
  postInstall = ''
    mkdir -p $out/conf
    cp template.json $out/conf/template.json
  '';
}
