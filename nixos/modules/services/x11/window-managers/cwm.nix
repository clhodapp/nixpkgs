{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.cwm;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../config/system-path.nix
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.cwm.enable = mkEnableOption "cwm";
  };
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "cwm";
      start = ''
        cwm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.cwm ];
  };
}
