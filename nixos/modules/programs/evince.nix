# Evince.

{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.programs.evince;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/system/dbus.nix
    ../system/boot/systemd.nix
    # END generated imports
  ];

  ###### interface

  options = {

    programs.evince = {

      enable = lib.mkEnableOption "Evince, the GNOME document viewer";

      package = lib.mkPackageOption pkgs "evince" { };

    };

  };

  ###### implementation

  config = lib.mkIf config.programs.evince.enable {

    environment.systemPackages = [ cfg.package ];

    services.dbus.packages = [ cfg.package ];

    systemd.packages = [ cfg.package ];

  };

}
