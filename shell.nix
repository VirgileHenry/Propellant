{ pkgs ? import <nixpkgs> {} }:

pkgs.mkShell {
  nativeBuildInputs = with pkgs; [
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
    # shader c build thingies
    cmake
    python3
  ];
  # this is hideous, please tell me how to do better
  LD_LIBRARY_PATH="${pkgs.libxkbcommon}/lib:${pkgs.libGL}/lib:${pkgs.wayland}/lib:${pkgs.vulkan-headers}/lib:${pkgs.vulkan-loader}/lib:${pkgs.vulkan-tools}/lib:${pkgs.vulkan-validation-layers}/lib:${pkgs.shaderc.bin}/lib:${pkgs.shaderc.static}/lib:${pkgs.shaderc.dev}/lib:${pkgs.shaderc.lib }/lib:${pkgs.cmake}/lib:${pkgs.python3}/lib";
}