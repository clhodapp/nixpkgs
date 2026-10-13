{ config, pkgs, ... }:
let
  apparmor = config.security.apparmor;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../apparmor.nix
    # END generated imports
  ];

  config.security.apparmor.packages = [ pkgs.apparmor-profiles ];
}
