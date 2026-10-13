# List all devices which are detected by nixos-generate-config.
# Common devices are enabled by default.
{ lib, ... }:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../hardware/all-firmware.nix
    # END generated imports
  ];

  config = lib.mkDefault {
    # Common firmware, i.e. for wifi cards
    hardware.enableRedistributableFirmware = true;
  };
}
