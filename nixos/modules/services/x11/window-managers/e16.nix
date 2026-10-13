{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.e16;
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
    services.xserver.windowManager.e16.enable = mkEnableOption "e16";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "E16";
      start = ''
        ${pkgs.e16}/bin/e16 &
        waitPID=$!
      '';
    };

    environment.systemPackages = [ pkgs.e16 ];
  };
}
