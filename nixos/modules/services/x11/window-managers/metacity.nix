{
  config,
  lib,
  pkgs,
  ...
}:

let

  cfg = config.services.xserver.windowManager.metacity;
  inherit (pkgs) metacity;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../config/system-path.nix
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.metacity.enable = lib.mkEnableOption "metacity";
  };

  config = lib.mkIf cfg.enable {

    services.xserver.windowManager.session = lib.singleton {
      name = "metacity";
      start = ''
        ${metacity}/bin/metacity &
        waitPID=$!
      '';
    };

    environment.systemPackages = [ metacity ];

  };

}
