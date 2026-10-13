# Enables non-free firmware on devices not recognized by `nixos-generate-config`.
{ lib, ... }:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../hardware/all-firmware.nix
    # END generated imports
  ];

  hardware.enableRedistributableFirmware = lib.mkDefault true;
}
