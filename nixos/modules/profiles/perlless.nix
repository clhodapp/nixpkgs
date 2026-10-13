{ lib, pkgs, ... }:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../installer/tools/tools.nix
    ../misc/documentation.nix
    ../programs/command-not-found/command-not-found.nix
    ../services/system/userborn.nix
    ../system/activation/top-level.nix
    ../system/boot/loader/grub/grub.nix
    ../system/etc/etc.nix
    # END generated imports
  ];

  # Remove perl from activation
  system.etc.overlay.enable = lib.mkDefault true;
  services.userborn.enable = lib.mkDefault true;

  # Random perl remnants
  system.tools.nixos-generate-config.enable = lib.mkDefault false;
  boot.loader.grub.enable = lib.mkDefault false;
  environment.defaultPackages = lib.mkDefault [ ];
  documentation.info.enable = lib.mkDefault false;
  documentation.nixos.enable = lib.mkDefault false;
  programs.command-not-found.enable = lib.mkDefault false;

  # Check that the system does not contain a Nix store path that contains the
  # string "perl".
  system.forbiddenDependenciesRegexes = [ "perl" ];
}
