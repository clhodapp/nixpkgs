{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.mwm;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  ###### interface
  options = {
    services.xserver.windowManager.mwm.enable = mkEnableOption "mwm";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "mwm";
      start = ''
        ${pkgs.motif}/bin/mwm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.motif ];
  };
}
