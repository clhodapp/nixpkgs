{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.cdemu;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../config/users-groups.nix
    ../services/hardware/udev.nix
    ../services/system/dbus.nix
    ../system/boot/kernel.nix
    ../system/boot/systemd/user.nix
    # END generated imports
  ];

  options = {
    programs.cdemu = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          {command}`cdemu` for members of
          {option}`programs.cdemu.group`.
        '';
      };
      group = lib.mkOption {
        type = lib.types.str;
        default = "cdrom";
        description = ''
          Group that users must be in to use {command}`cdemu`.
        '';
      };
      gui = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = ''
          Whether to install the {command}`cdemu` GUI (gCDEmu).
        '';
      };
      image-analyzer = lib.mkOption {
        type = lib.types.bool;
        default = true;
        description = ''
          Whether to install the image analyzer.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {

    boot = {
      extraModulePackages = [ config.boot.kernelPackages.vhba ];
      kernelModules = [ "vhba" ];
    };

    services = {
      udev.extraRules = ''
        KERNEL=="vhba_ctl", MODE="0660", OWNER="root", GROUP="${cfg.group}"
      '';
      dbus.packages = [ pkgs.cdemu-daemon ];
    };

    users.groups.${config.programs.cdemu.group} = { };

    # Systemd User service
    # manually adapted from example in source package:
    # https://sourceforge.net/p/cdemu/code/ci/master/tree/cdemu-daemon/service-example/cdemu-daemon.service
    systemd.user.services.cdemu-daemon.description = "CDEmu daemon";
    systemd.user.services.cdemu-daemon.serviceConfig = {
      Type = "dbus";
      BusName = "net.sf.cdemu.CDEmuDaemon";
      ExecStart = "${lib.getExe pkgs.cdemu-daemon} --config-file \"%h/.config/cdemu-daemon\"";
      Restart = "no";
    };

    environment.systemPackages = [
      pkgs.cdemu-daemon
      pkgs.cdemu-client
    ]
    ++ lib.optional cfg.gui pkgs.gcdemu
    ++ lib.optional cfg.image-analyzer pkgs.image-analyzer;
  };

}
