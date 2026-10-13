{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.qdmr;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../config/users-groups.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  meta.maintainers = [ ];

  options = {
    programs.qdmr = {
      enable = lib.mkEnableOption "QDMR - a GUI application and command line tool for programming DMR radios";
      package = lib.mkPackageOption pkgs "qdmr" { };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.udev.packages = [ cfg.package ];
    users.groups.dialout = { };
  };
}
