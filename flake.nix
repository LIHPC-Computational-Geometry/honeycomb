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
        lib = pkgs.lib;
        
        craneLib = (crane.mkLib pkgs).overrideToolchain (p: p.fenix.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ]
        );
        # src = craneLib.cleanCargoSource ./.;
        unfilteredRoot = ./.;
        src = lib.fileset.toSource {
          root = unfilteredRoot;
          fileset = lib.fileset.unions [
            # Default files from crane (Rust and cargo files)
            (craneLib.fileset.commonCargoSources unfilteredRoot)
            # Also keep any VTK files, this is a dirty fix for tests which use our example vtk file
            # TODO: VTK files should be excluded to avoid indexing of residual output files
            (lib.fileset.fileFilter (file: file.hasExt "vtk") unfilteredRoot)
          ];
        };

        commonBuildInputs = with pkgs; [
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

          nativeBuildInputs = with pkgs; [
            pkg-config
          ];
          buildInputs = commonBuildInputs
            ++ (if pkgs.stdenv.isLinux  then linuxBuildInputs  else [])
            ++ (if pkgs.stdenv.isDarwin then darwinBuildInputs else []);
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

          # Lints
          honeycomb-clippy = craneLib.cargoClippy (
            commonArgs
            // {
              inherit cargoArtifacts;
              cargoClippyExtraArgs = "--all-targets -- --deny warnings";
            }
          );

          # Format
          honeycomb-fmt = craneLib.cargoFmt {
            inherit src;
          };
          honeycomb-toml-fmt = craneLib.taploFmt {
            src = pkgs.lib.sources.sourceFilesBySuffices src [ ".toml" ];
            taploExtraArgs = "--config ./.taplo.toml";
          };

          # Test
          honeycomb-doctest = craneLib.cargoDocTest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
          honeycomb-test = craneLib.cargoNextest (
            commonArgs
            // {
              inherit cargoArtifacts;
            }
          );
        };
        
        devShells.default = craneLib.devShell {
          checks = self.checks.${system};
          
          packages = with pkgs; [
            cargo-nextest # faster tests
            samply        # profiling
            taplo         # TOML formatting
          ];
        };
      });
}
