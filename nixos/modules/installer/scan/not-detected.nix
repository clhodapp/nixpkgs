# Enables non-free firmware on devices not recognized by `nixos-generate-config`.
{ lib, ... }:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../hardware/all-firmware.nix
    # END generated imports
  ];

  hardware.enableRedistributableFirmware = lib.mkDefault true;
}
