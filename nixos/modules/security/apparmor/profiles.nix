{ config, pkgs, ... }:
let
  apparmor = config.security.apparmor;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../apparmor.nix
    # END generated imports
  ];

  config.security.apparmor.packages = [ pkgs.apparmor-profiles ];
}
