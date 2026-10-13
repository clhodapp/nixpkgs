{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let

  cfg = config.services.xserver.windowManager.twm;

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
    services.xserver.windowManager.twm.enable = mkEnableOption "twm";
  };

  ###### implementation

  config = mkIf cfg.enable {

    services.xserver.windowManager.session = singleton {
      name = "twm";
      start = ''
        ${pkgs.tab-window-manager}/bin/twm &
        waitPID=$!
      '';
    };

    environment.systemPackages = [ pkgs.tab-window-manager ];

  };

}
