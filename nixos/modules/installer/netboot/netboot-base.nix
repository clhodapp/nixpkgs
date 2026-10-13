# This module contains the basic configuration for building netboot
# images

{ lib, ... }:

{
  imports = [
    ./netboot.nix

    # Profiles of this basic netboot media
    ../../profiles/base.nix
    ../../profiles/installation-device.nix
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../hardware/all-hardware.nix
    # END generated imports
  ];

  hardware.enableAllHardware = true;
}
