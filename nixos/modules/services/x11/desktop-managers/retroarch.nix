{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.desktopManager.retroarch;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../../lib/modules/generic/meta-maintainers.nix
    ../../../config/system-path.nix
    ./default.nix
    # END generated imports
  ];

  options.services.xserver.desktopManager.retroarch = {
    enable = mkEnableOption "RetroArch";

    package = mkPackageOption pkgs "retroarch" {
      example = "retroarch-full";
    };

    extraArgs = mkOption {
      type = types.listOf types.str;
      default = [ ];
      example = [
        "--verbose"
        "--host"
      ];
      description = "Extra arguments to pass to RetroArch.";
    };
  };

  config = mkIf cfg.enable {
    services.xserver.desktopManager.session = [
      {
        name = "RetroArch";
        start = ''
          ${cfg.package}/bin/retroarch -f ${escapeShellArgs cfg.extraArgs} &
          waitPID=$!
        '';
      }
    ];

    environment.systemPackages = [ cfg.package ];
  };

  meta.maintainers = with maintainers; [ j0hax ];
}
