{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.smallwm;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../config/system-path.nix
    ./default.nix
    # END generated imports
  ];

  ###### interface
  options = {
    services.xserver.windowManager.smallwm.enable = mkEnableOption "smallwm";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "smallwm";
      start = ''
        ${pkgs.smallwm}/bin/smallwm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.smallwm ];
  };
}
