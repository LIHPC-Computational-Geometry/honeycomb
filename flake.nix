{
  inputs = {
    fenix = {
      url = "github:nix-community/fenix";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "nixpkgs/nixos-unstable";
  };

  outputs = { nixpkgs, flake-utils, fenix, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ fenix.overlays.default ];
        pkgs = import nixpkgs { inherit system overlays; };

        
        commonBuildInputs = with pkgs; [
          # Rust
          (fenix.packages.${system}.stable.withComponents [
            "cargo"
            "clippy"
            "rust-src"
            "rustc"
            "rustfmt"
          ])

          # Deps
          hwloc.dev

          # Tools
          cargo-nextest
          samply
          taplo
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

        buildInputs = commonBuildInputs
          ++ (if pkgs.stdenv.isLinux  then linuxBuildInputs  else [])
          ++ (if pkgs.stdenv.isDarwin then darwinBuildInputs else []);

        ldLibraryPath = pkgs.lib.makeLibraryPath ( buildInputs );
      in {
        devShell = pkgs.mkShell {
          buildInputs = buildInputs;
          
          LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${ldLibraryPath}";

          # TODO: make this work
          # shellHook = ''
          #   $SHELL
          # '';
        };
      });
}
