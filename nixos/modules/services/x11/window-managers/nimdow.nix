{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.nimdow;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.windowManager.nimdow.enable = mkEnableOption "nimdow";
    services.xserver.windowManager.nimdow.package = mkPackageOption pkgs "nimdow" { };
  };

  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "nimdow";
      start = ''
        ${cfg.package}/bin/nimdow &
        waitPID=$!
      '';
    };
    environment.systemPackages = [
      cfg.package
      pkgs.st
    ];
  };
}
