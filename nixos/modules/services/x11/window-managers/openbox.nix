{
  lib,
  pkgs,
  config,
  ...
}:

with lib;
let
  cfg = config.services.xserver.windowManager.openbox;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../config/system-path.nix
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.openbox.enable = mkEnableOption "openbox";
  };

  config = mkIf cfg.enable {
    services.xserver.windowManager = {
      session = [
        {
          name = "openbox";
          start = "
          ${pkgs.openbox}/bin/openbox-session
        ";
        }
      ];
    };
    environment.systemPackages = [ pkgs.openbox ];
  };
}
