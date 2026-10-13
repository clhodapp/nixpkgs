{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.berry;
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
    services.xserver.windowManager.berry.enable = mkEnableOption "berry";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "berry";
      start = ''
        ${pkgs.berry}/bin/berry &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.berry ];
  };
}
