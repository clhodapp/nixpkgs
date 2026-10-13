{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.playerctld;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../system/boot/systemd/user.nix
    # END generated imports
  ];

  options.services.playerctld = {
    enable = lib.mkEnableOption "the playerctld daemon";

    package = lib.mkPackageOption pkgs "playerctl" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    systemd.user.services.playerctld = {
      description = "Playerctld daemon to track media player activity";
      wantedBy = [ "default.target" ];

      serviceConfig = {
        Type = "exec";
        ExecStart = "${cfg.package}/bin/playerctld";
      };
    };
  };

  meta.maintainers = with lib.maintainers; [ aacebedo ];
}
