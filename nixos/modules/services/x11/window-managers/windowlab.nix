{
  lib,
  pkgs,
  config,
  ...
}:

let
  cfg = config.services.xserver.windowManager.windowlab;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../config/system-path.nix
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.windowlab.enable = lib.mkEnableOption "windowlab";
  };

  config = lib.mkIf cfg.enable {
    services.xserver.windowManager = {
      session = [
        {
          name = "windowlab";
          start = "${pkgs.windowlab}/bin/windowlab";
        }
      ];
    };
    environment.systemPackages = [ pkgs.windowlab ];
  };
}
