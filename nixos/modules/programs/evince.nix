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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/system/dbus.nix
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
