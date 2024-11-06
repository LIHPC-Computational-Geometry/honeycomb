{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  buildInputs = with pkgs; [
    # Rust
    rustup
    # Render tool libs
    xorg.libX11
    xorg.libXcursor
    xorg.libXi
    libxkbcommon
    xorg.libxcb
    libudev-zero
  ];

  # Render tool libs
  LD_LIBRARY_PATH = "$LD_LIBRARY_PATH:${
    with pkgs;
    pkgs.lib.makeLibraryPath [
      xorg.libX11
      xorg.libXcursor
      xorg.libXi
      libxkbcommon
      xorg.libxcb
      pkgs.vulkan-loader
      pkgs.glfw
    ]
  }";

  # Shell hook to set up the environment
  shellHook = ''
    # Initialize rustup if not already done
    if [ ! -d "$HOME/.rustup" ]; then
      rustup default stable
      rustup component add rust-analyzer
    fi
  '';
}
