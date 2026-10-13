# Set `hardware.enableAllHardware = true` instead of importing this file.
# This file provides backwards compatibility for older configurations.
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../hardware/all-hardware.nix
    # END generated imports
  ];

  hardware.enableAllHardware = true;
}
