{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.evilwm;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  ###### interface
  options = {
    services.xserver.windowManager.evilwm.enable = mkEnableOption "evilwm";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "evilwm";
      start = ''
        ${pkgs.evilwm}/bin/evilwm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.evilwm ];
  };
}
