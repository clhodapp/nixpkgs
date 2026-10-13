{ lib, ... }:

with lib;

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../boot/loader/grub/grub.nix
    ./specialisation.nix
    # END generated imports
  ];

  boot.loader.grub.device = mkOverride 0 "nodev";
  specialisation = mkOverride 0 { };
  isSpecialisation = mkOverride 0 true;
}
