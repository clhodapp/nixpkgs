{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.xserver.windowManager.dk;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../config/system-path.nix
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.dk = {
      enable = lib.mkEnableOption "dk";

      package = lib.mkPackageOption pkgs "dk" { };
    };
  };

  config = lib.mkIf cfg.enable {
    services.xserver.windowManager.session = lib.singleton {
      name = "dk";
      start = ''
        export _JAVA_AWT_WM_NONREPARENTING=1
        ${cfg.package}/bin/dk &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ cfg.package ];
  };
}
