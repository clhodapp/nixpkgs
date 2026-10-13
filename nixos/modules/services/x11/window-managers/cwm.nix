{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.cwm;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.cwm.enable = mkEnableOption "cwm";
  };
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "cwm";
      start = ''
        cwm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.cwm ];
  };
}
