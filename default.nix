{ pkgs, deps, ... }: {
  game = pkgs.rustPlatform.buildRustPackage {
    pname = "vitriol";
    version = "0.0.1";
    src = ./.;
    cargoBuildFlags = "";

    cargoLock = {
      lockFile = ./Cargo.lock;
    };

    nativeBuildInputs = deps;
  };
}
