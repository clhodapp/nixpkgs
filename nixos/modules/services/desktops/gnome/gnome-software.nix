{
  config,
  pkgs,
  lib,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../../lib/modules/generic/meta-maintainers.nix
    ../../../config/system-path.nix
    ../../../system/boot/systemd.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.gnome ];
  };

  options = {
    services.gnome.gnome-software = {
      enable = lib.mkEnableOption "GNOME Software, package manager for GNOME";
    };
  };

  config = lib.mkIf config.services.gnome.gnome-software.enable {
    environment.systemPackages = [
      pkgs.gnome-software
    ];

    systemd.packages = [
      pkgs.gnome-software
    ];
  };
}
