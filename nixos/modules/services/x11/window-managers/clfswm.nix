{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.clfswm;
in

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.clfswm = {
      enable = mkEnableOption "clfswm";
      package = mkPackageOption pkgs [ "sbclPackages" "clfswm" ] { };
    };
  };

  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "clfswm";
      start = ''
        ${cfg.package}/bin/clfswm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ cfg.package ];
  };
}
