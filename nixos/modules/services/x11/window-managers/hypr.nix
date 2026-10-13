{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.hypr;
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
    services.xserver.windowManager.hypr.enable = mkEnableOption "hypr";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "hypr";
      start = ''
        ${pkgs.hypr}/bin/Hypr &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.hypr ];
  };
}
