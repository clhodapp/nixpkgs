{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.programs.flexoptix-app;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options = {
    programs.flexoptix-app = {
      enable = lib.mkEnableOption "FLEXOPTIX app + udev rules";

      package = lib.mkPackageOption pkgs "flexoptix-app" { };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.udev.packages = [ cfg.package ];
  };
}
