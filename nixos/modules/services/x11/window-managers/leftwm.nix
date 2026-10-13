{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.leftwm;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  ###### interface
  options = {
    services.xserver.windowManager.leftwm.enable = mkEnableOption "leftwm";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "leftwm";
      start = ''
        ${pkgs.leftwm}/bin/leftwm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.leftwm ];
  };
}
