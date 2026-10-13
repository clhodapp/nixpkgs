{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.tinywm;
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
    services.xserver.windowManager.tinywm.enable = mkEnableOption "tinywm";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "tinywm";
      start = ''
        ${pkgs.tinywm}/bin/tinywm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.tinywm ];
  };
}
