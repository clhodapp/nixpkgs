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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
