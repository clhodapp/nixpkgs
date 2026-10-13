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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
