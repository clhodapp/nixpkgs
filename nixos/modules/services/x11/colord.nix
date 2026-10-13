{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let

  cfg = config.services.colord;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../config/users-groups.nix
    ../../system/boot/systemd.nix
    ../../system/boot/systemd/tmpfiles.nix
    ../hardware/udev.nix
    ../system/dbus.nix
    # END generated imports
  ];

  options = {

    services.colord = {
      enable = mkEnableOption "colord, the color management daemon";
    };

  };

  config = mkIf cfg.enable {

    environment.systemPackages = [ pkgs.colord ];

    services.dbus.packages = [ pkgs.colord ];

    services.udev.packages = [ pkgs.colord ];

    systemd.packages = [ pkgs.colord ];

    systemd.tmpfiles.packages = [ pkgs.colord ];

    users.users.colord = {
      isSystemUser = true;
      home = "/var/lib/colord";
      group = "colord";
    };

    users.groups.colord = { };

  };

}
