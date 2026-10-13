{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.projecteur;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.programs.projecteur = {
    enable = lib.mkEnableOption "projecteur, an application for the Logitech Spotlight device (and similar)";
    package = lib.mkPackageOption pkgs "projecteur" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.udev.packages = [ cfg.package ];
  };

  meta = {
    maintainers = with lib.maintainers; [
      benneti
    ];
  };
}
