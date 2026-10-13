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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
