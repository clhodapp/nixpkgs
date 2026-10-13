{
  config,
  pkgs,
  lib,
  ...
}:

let
  inherit (lib) mkEnableOption mkIf mkPackageOption;
  cfg = config.services.realmd;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../system/dbus.nix
    # END generated imports
  ];

  options.services.realmd = {
    enable = mkEnableOption "realmd service for managing system enrollment in Active Directory domains";

    package = mkPackageOption pkgs "realmd" { };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    services.dbus = {
      enable = true;
      packages = [ cfg.package ];
    };

    systemd.services.realmd = {
      description = "Realm and Domain Configuration";
      wantedBy = [ "multi-user.target" ];
      partOf = [ "dbus.service" ];
      requires = [ "dbus.service" ];
      after = [
        "network.target"
        "dbus.service"
      ];
      serviceConfig = {
        Type = "dbus";
        BusName = "org.freedesktop.realmd";
        ExecStart = "${cfg.package}/libexec/realmd";
        RuntimeDirectory = "realmd";
      };
    };
  };
}
