{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.inputplumber;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options.services.inputplumber = {
    enable = lib.mkEnableOption "InputPlumber";
    package = lib.mkPackageOption pkgs "inputplumber" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    environment.pathsToLink = [ "/share/inputplumber" ];

    systemd.services.inputplumber = {
      description = "InputPlumber Service";
      wantedBy = [ "multi-user.target" ];
      environment = {
        XDG_DATA_DIRS = "/run/current-system/sw/share";
      };
      restartIfChanged = true;

      serviceConfig = {
        ExecStart = "${lib.getExe cfg.package}";
        Restart = "on-failure";
        RestartSec = "5";
      };
    };
  };

  meta.maintainers = with lib.maintainers; [ shadowapex ];
}
