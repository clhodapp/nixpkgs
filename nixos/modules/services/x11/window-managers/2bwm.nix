{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let

  cfg = config.services.xserver.windowManager."2bwm";

in

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  ###### interface

  options = {
    services.xserver.windowManager."2bwm".enable = mkEnableOption "2bwm";
  };

  ###### implementation

  config = mkIf cfg.enable {

    services.xserver.windowManager.session = singleton {
      name = "2bwm";
      start = ''
        ${pkgs._2bwm}/bin/2bwm &
        waitPID=$!
      '';
    };

    environment.systemPackages = [ pkgs._2bwm ];

  };

}
