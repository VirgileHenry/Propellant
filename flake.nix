{
  inputs = {
    flake-utils.url = "github:numtide/flake-utils";
    nixpkgs.url = "github:nixos/nixpkgs?ref=release-21.11";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    ...
  }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs { inherit system; };
      in with pkgs; rec {
        devShell = mkShell rec {
          buildInputs = [
            libxkbcommon
            libGL

            # WINIT_UNIX_BACKEND=wayland
            wayland

            # pure vulkan stuff
            pkgs.vulkan-headers
            pkgs.vulkan-loader
            pkgs.vulkan-tools
            pkgs.vulkan-validation-layers

            # shaderc: compile glsl to risc-v
            pkgs.shaderc
            pkgs.shaderc.bin
            pkgs.shaderc.static
            pkgs.shaderc.dev
            pkgs.shaderc.lib    
          ];
          LD_LIBRARY_PATH="${pkgs.vulkan-loader}/lib:${pkgs.shaderc.lib}/lib:${pkgs.shaderc.dev}/lib";
          VULKAN_LIB_DIR="${pkgs.shaderc.dev}/lib";
          RUST_BACKTRACE=1;
          CARGO_PROFILE_DEV_BUILD_OVERRIDE_DEBUG=true;
        };
      });
}