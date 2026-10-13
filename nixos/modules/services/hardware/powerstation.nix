{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.powerstation;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options.services.powerstation = {
    enable = lib.mkEnableOption "PowerStation";
    package = lib.mkPackageOption pkgs "powerstation" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    systemd.services.powerstation = {
      description = "PowerStation Service";
      wantedBy = [ "multi-user.target" ];
      after = [ "graphical-session.target" ];
      environment = {
        XDG_DATA_DIRS = "/run/current-system/sw/share";
      };

      serviceConfig = {
        User = "root";
        Group = "root";
        ExecStart = lib.getExe cfg.package;
      };
    };
  };

  meta.maintainers = with lib.maintainers; [ shadowapex ];
}
