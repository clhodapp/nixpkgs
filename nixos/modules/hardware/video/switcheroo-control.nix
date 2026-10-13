{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.services.switcherooControl;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../services/system/dbus.nix
    # END generated imports
  ];

  options.services.switcherooControl = {
    enable = lib.mkEnableOption "switcheroo-control, a D-Bus service to check the availability of dual-GPU";
    package = lib.mkPackageOption pkgs "switcheroo-control" { };
  };

  config = lib.mkIf cfg.enable {
    services.dbus.packages = [ cfg.package ];
    environment.systemPackages = [ cfg.package ];
    systemd = {
      packages = [ cfg.package ];
      targets.multi-user.wants = [ "switcheroo-control.service" ];
    };
  };
}
