{ lib, ... }:

with lib;

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./specialisation.nix
    # END generated imports
  ];

  boot.loader.grub.device = mkOverride 0 "nodev";
  specialisation = mkOverride 0 { };
  isSpecialisation = mkOverride 0 true;
}
