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
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
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
