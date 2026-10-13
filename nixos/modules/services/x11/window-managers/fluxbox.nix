{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.fluxbox;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  ###### interface
  options = {
    services.xserver.windowManager.fluxbox.enable = mkEnableOption "fluxbox";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "fluxbox";
      start = ''
        ${pkgs.fluxbox}/bin/startfluxbox &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.fluxbox ];
  };
}
