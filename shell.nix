{ pkgs ? import <nixpkgs> {} }:
pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
    cmake
    pkg-config
    python3
  ];

  buildInputs = with pkgs; [
    libxkbcommon
    libGL
    wayland
    vulkan-headers
    vulkan-loader
    vulkan-tools
    vulkan-validation-layers
    shaderc
  ];

  SHADERC_LIB_DIR = "${pkgs.shaderc.lib}/lib";

  LD_LIBRARY_PATH = pkgs.lib.makeLibraryPath (with pkgs; [
    libxkbcommon
    libGL
    wayland
    vulkan-loader
    shaderc.lib
  ]);
}
