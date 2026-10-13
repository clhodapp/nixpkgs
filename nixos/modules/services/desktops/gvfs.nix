# GVfs

{
  config,
  lib,
  pkgs,
  ...
}:

let

  cfg = config.services.gvfs;

in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-environment.nix
    ../../config/system-path.nix
    ../../programs/fuse.nix
    ../../system/boot/systemd.nix
    ../hardware/udev.nix
    ../hardware/udisks2.nix
    ../system/dbus.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.gnome ];
  };

  ###### interface

  options = {

    services.gvfs = {

      enable = lib.mkEnableOption "GVfs, a userspace virtual filesystem";

      # gvfs can be built with multiple configurations
      package = lib.mkPackageOption pkgs [ "gnome" "gvfs" ] { };

    };

  };

  ###### implementation

  config = lib.mkIf cfg.enable {

    environment.systemPackages = [ cfg.package ];

    programs.fuse.enable = true;

    services.dbus.packages = [ cfg.package ];

    systemd.packages = [ cfg.package ];

    services.udev.packages = [ pkgs.libmtp.out ];

    services.udisks2.enable = true;

    # Needed for unwrapped applications
    environment.sessionVariables.GIO_EXTRA_MODULES = [ "${cfg.package}/lib/gio/modules" ];

  };

}
