{
  inputs = {
    crane.url = "github:ipetkov/crane";
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { self, nixpkgs, flake-utils, fenix, crane, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };
        
        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.fenix.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ]
        );
        src = craneLib.cleanCargoSource ./.;

        commonBuildInputs = with pkgs; [
          pkg-config
          hwloc.dev
        ];
        linuxBuildInputs = with pkgs; [
          xorg.libX11
          xorg.libXcursor
          xorg.libXi
          libxkbcommon
          xorg.libxcb
          libudev-zero
          vulkan-loader
          glfw
        ];
        darwinBuildInputs = with pkgs; [
          libiconv
        ];

        commonArgs = {
          inherit src;
          strictDeps = true;

          buildInputs = commonBuildInputs
            ++ (if pkgs.stdenv.isLinux  then linuxBuildInputs  else [])
            ++ (if pkgs.stdenv.isDarwin then darwinBuildInputs else []);
          LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
            pkgs.lib.makeLibraryPath ( commonBuildInputs
            ++ (if pkgs.stdenv.isLinux  then linuxBuildInputs  else [])
            ++ (if pkgs.stdenv.isDarwin then darwinBuildInputs else []) )
          }";
        };

        cargoArtifacts = craneLib.buildDepsOnly commonArgs;

        honeycomb = craneLib.buildPackage (
          commonArgs
          // {
            inherit cargoArtifacts;
          }
        );
      in {
        checks = {
          inherit honeycomb;

          honeycomb-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );
        };
        
        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          
          packages = with pkgs; [
            cargo-nextest
            samply
            taplo
          ];

          # TODO: make this work
          # shellHook = ''
          #   $SHELL
          # '';
        };
      });
}
