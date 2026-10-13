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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
