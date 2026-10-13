# This module defines a small netboot environment.

{ lib, ... }:

{
  imports = [
    ./netboot-base.nix
    ../../profiles/minimal.nix
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../hardware/all-firmware.nix
    ../../misc/documentation.nix
    ../../services/networking/networkmanager.nix
    ../../system/activation/top-level.nix
    # END generated imports
  ];

  documentation.man.enable = lib.mkOverride 500 true;
  hardware.enableRedistributableFirmware = lib.mkOverride 70 false;
  system.extraDependencies = lib.mkOverride 70 [ ];
  networking.networkmanager.enable = lib.mkOverride 500 false;
}
