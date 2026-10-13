# List all devices which are detected by nixos-generate-config.
# Common devices are enabled by default.
{ lib, ... }:
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../hardware/all-firmware.nix
    # END generated imports
  ];

  config = lib.mkDefault {
    # Common firmware, i.e. for wifi cards
    hardware.enableRedistributableFirmware = true;
  };
}
