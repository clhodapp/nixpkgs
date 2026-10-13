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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
