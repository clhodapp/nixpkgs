{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.lwm;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  ###### interface
  options = {
    services.xserver.windowManager.lwm.enable = mkEnableOption "lwm";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "lwm";
      start = ''
        ${pkgs.lwm}/bin/lwm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.lwm ];
  };
}
