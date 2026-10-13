{
  lib,
  pkgs,
  config,
  ...
}:
let
  cfg = config.programs.kclock;
  kclockPkg = pkgs.kdePackages.kclock;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/system/dbus.nix
    # END generated imports
  ];

  options.programs.kclock = {
    enable = lib.mkEnableOption "KClock";
  };

  config = lib.mkIf cfg.enable {
    services.dbus.packages = [ kclockPkg ];
    environment.systemPackages = [ kclockPkg ];
  };
}
