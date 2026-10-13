{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.xserver.windowManager.oxwm;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../config/system-path.nix
    ../../display-managers/default.nix
    # END generated imports
  ];

  options.services.xserver.windowManager.oxwm = {
    enable = lib.mkEnableOption "oxwm";
    package = lib.mkPackageOption pkgs "oxwm" { };
  };

  config = lib.mkIf cfg.enable {
    services.displayManager.sessionPackages = [ cfg.package ];
    environment.systemPackages = [ cfg.package ];
    environment.pathsToLink = [
      "/share/oxwm"
      "/share/xsessions"
    ];
  };
}
