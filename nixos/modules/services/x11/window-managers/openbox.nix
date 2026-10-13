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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
