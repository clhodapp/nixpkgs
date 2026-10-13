{ pkgs, ... }:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../services/hardware/udev.nix
    # END generated imports
  ];

  hardware.firmware = [ pkgs.zd1211fw ];
}
