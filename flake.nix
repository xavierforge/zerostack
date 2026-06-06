{
  description = "Minimalistic coding agent written in Rust, optimized for memory footprint and performance";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = nixpkgs.legacyPackages.${system};
      in
      {
        packages.default = pkgs.rustPlatform.buildRustPackage {
          pname = "zerostack";
          version = "1.4.5";
          src = self;
          cargoLock.lockFile = ./Cargo.lock;

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];

          buildInputs = with pkgs; [
            openssl
          ];

          buildFeatures = [ "loop" "git-worktree" "mcp" "subagents" "archmd" ];
        };

        apps.default = {
          type = "app";
          program = "${self.packages.${system}.default}/bin/zerostack";
        };
      }
    );
}
