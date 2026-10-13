{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.notion;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../config/system-path.nix
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.notion.enable = mkEnableOption "notion";
  };

  config = mkIf cfg.enable {
    services.xserver.windowManager = {
      session = [
        {
          name = "notion";
          start = ''
            ${pkgs.notion}/bin/notion &
            waitPID=$!
          '';
        }
      ];
    };
    environment.systemPackages = [ pkgs.notion ];
  };
}
