{
  description = "Fastiron Nix flake with Rust and hwloc support";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    rust-overlay.url = "github:oxalica/rust-overlay";
  };

  outputs = { self, nixpkgs, flake-utils, rust-overlay }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
          overlays = [ rust-overlay.overlays.default ];
        };

        # Rust toolchain with extensions
        rustToolchain = pkgs.rust-bin.stable.latest.default.override {
          extensions = [ "rust-src" "rust-analyzer" ];
        };

        # Platform-specific build inputs
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

        buildInputs = with pkgs; [
          hwloc.dev
        ] ++ (if pkgs.stdenv.isLinux then linuxBuildInputs else []);

        # Platform-specific LD_LIBRARY_PATH
        ldLibraryPath = pkgs.lib.makeLibraryPath (
          [ pkgs.hwloc.lib ] ++
          (if pkgs.stdenv.isLinux then linuxBuildInputs else [])
        );

      in
      {
        devShells.default = pkgs.mkShell {
          nativeBuildInputs = with pkgs; [
            rustToolchain
            cargo
            rust-analyzer
            pkg-config
          ];

          buildInputs = buildInputs;

          shellHook = ''
            export LD_LIBRARY_PATH=${ldLibraryPath}:$LD_LIBRARY_PATH
            # Initialize rustup if not already done
            if [ ! -d "$HOME/.rustup" ]; then
              rustup default stable
              rustup component add rust-analyzer
            fi
          '';
        };
      });
}
