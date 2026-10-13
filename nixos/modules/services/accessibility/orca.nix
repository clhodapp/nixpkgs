{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.orca;
  inherit (lib)
    mkEnableOption
    mkIf
    mkPackageOption
    ;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../desktops/gnome/at-spi2-core.nix
    # END generated imports
  ];

  options.services.orca = {
    enable = mkEnableOption "Orca screen reader";
    package = mkPackageOption pkgs "orca" { };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    systemd.packages = [ cfg.package ];
    services.gnome.at-spi2-core.enable = true;
    services.speechd.enable = true;
  };
}
