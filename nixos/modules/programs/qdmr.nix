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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../config/users-groups.nix
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
