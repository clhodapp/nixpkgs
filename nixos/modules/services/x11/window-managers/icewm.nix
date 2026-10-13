{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.icewm;
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
    services.xserver.windowManager.icewm.enable = mkEnableOption "icewm";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "icewm";
      start = ''
        ${pkgs.icewm}/bin/icewm-session &
        waitPID=$!
      '';
    };

    environment.systemPackages = [ pkgs.icewm ];
  };
}
