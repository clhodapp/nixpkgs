{ pkgs, ... }:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../services/hardware/udev.nix
    # END generated imports
  ];

  hardware.firmware = [ pkgs.zd1211fw ];
}
