{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.stumpwm;
in

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.stumpwm.enable = mkEnableOption "stumpwm";
    services.xserver.windowManager.stumpwm.package = mkPackageOption pkgs "stumpwm" { };
  };

  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "stumpwm";
      start = ''
        ${cfg.package}/bin/stumpwm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ cfg.package ];
  };
}
