{
  lib,
  pkgs,
  config,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.vxwm;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.vxwm.enable = mkEnableOption "vxwm";
  };

  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "vxwm";
      start = ''
        ${pkgs.vxwm}/bin/vxwm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.vxwm ];
  };
}
