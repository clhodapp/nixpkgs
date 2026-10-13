{
  config,
  pkgs,
  lib,
  ...
}:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/systemd.nix
    ../hardware/udev.nix
    ../system/dbus.nix
    # END generated imports
  ];

  ###### interface

  options = {

    services.system-config-printer = {

      enable = lib.mkEnableOption "system-config-printer, a service for CUPS administration used by printing interfaces";

    };

  };

  ###### implementation

  config = lib.mkIf config.services.system-config-printer.enable {

    services.dbus.packages = [
      pkgs.system-config-printer
    ];

    systemd.packages = [
      pkgs.system-config-printer
    ];

    services.udev.packages = [
      pkgs.system-config-printer
    ];

    # for $out/bin/install-printer-driver
    # TODO: Enable once #177946 is resolved
    # services.packagekit.enable = true;

  };

}
