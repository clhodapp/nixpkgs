# This module defines a small NixOS configuration.  It does not
# contain any graphical stuff.

{
  lib,
  ...
}:
let
  inherit (lib) mkDefault;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../config/stub-ld.nix
    ../programs/command-not-found/command-not-found.nix
    # END generated imports
  ];

  documentation = {
    enable = mkDefault false;
    doc.enable = mkDefault false;
    info.enable = mkDefault false;
    man.enable = mkDefault false;
    nixos.enable = mkDefault false;
  };

  environment = {
    # Perl is a default package.
    defaultPackages = mkDefault [ ];
    stub-ld.enable = mkDefault false;
  };

  programs = {
    command-not-found.enable = mkDefault false;
    fish.generateCompletions = mkDefault false;
  };

  services = {
    logrotate.enable = mkDefault false;
    udisks2.enable = mkDefault false;
  };

  xdg = {
    autostart.enable = mkDefault false;
    icons.enable = mkDefault false;
    mime.enable = mkDefault false;
    sounds.enable = mkDefault false;
  };
}
