{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.bazecor;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options = {
    programs.bazecor = {
      enable = lib.mkEnableOption "Bazecor, the graphical configurator for Dygma Products";
      package = lib.mkPackageOption pkgs "bazecor" { };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.udev.packages = [ cfg.package ];
  };
}
