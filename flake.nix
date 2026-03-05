{
  description = "nix dev environment for winit + wgpu";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs { inherit system overlays; };

        # Libraries needed at runtime by winit + wgpu
        runtimeLibs = with pkgs; [
          wayland
          libxkbcommon
          vulkan-loader

          # X11 fallback
          libx11
          libxcursor
          libxrandr
          libxi
        ];
      in
      {
        devShells.default = pkgs.mkShell {
          buildInputs = with pkgs; [
            pkg-config

            # Wayland
            wayland
            wayland-protocols
            wayland-scanner
            libxkbcommon

            # Vulkan / wgpu
            vulkan-loader
            vulkan-headers
            vulkan-validation-layers
            vulkan-tools

            # X11 fallback
            libx11
            libxcursor
            libxrandr
            libxi
          ];

          LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath runtimeLibs;
        };
      }
    );
}
