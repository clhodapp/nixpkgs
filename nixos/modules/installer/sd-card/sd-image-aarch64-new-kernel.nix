{ pkgs, ... }:

{
  imports = [
    ./sd-image-aarch64.nix
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/kernel.nix
    # END generated imports
  ];

  boot.kernelPackages = pkgs.linuxPackages_latest;
}
