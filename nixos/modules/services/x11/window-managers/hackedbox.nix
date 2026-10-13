{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.hackedbox;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  ###### interface
  options = {
    services.xserver.windowManager.hackedbox.enable = mkEnableOption "hackedbox";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "hackedbox";
      start = ''
        ${pkgs.hackedbox}/bin/hackedbox &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.hackedbox ];
  };
}
