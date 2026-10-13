{ pkgs, ... }:

{
  imports = [
    ./installation-cd-minimal.nix
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/kernel.nix
    # END generated imports
  ];

  boot.kernelPackages = pkgs.linuxPackages_latest;
}
