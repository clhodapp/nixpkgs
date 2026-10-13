{
  lib,
  pkgs,
  config,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.vxwm;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../config/system-path.nix
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.vxwm.enable = mkEnableOption "vxwm";
  };

  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "vxwm";
      start = ''
        ${pkgs.vxwm}/bin/vxwm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.vxwm ];
  };
}
