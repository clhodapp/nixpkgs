# Set `hardware.enableAllHardware = true` instead of importing this file.
# This file provides backwards compatibility for older configurations.
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../hardware/all-hardware.nix
    # END generated imports
  ];

  hardware.enableAllHardware = true;
}
