{
  config,
  pkgs,
  lib,
  ...
}:

{
  meta = {
    teams = [ lib.teams.gnome ];
  };

  imports = [
    (lib.mkRenamedOptionModule
      [
        "services"
        "gnome"
        "tracker-miners"
        "enable"
      ]
      [
        "services"
        "gnome"
        "localsearch"
        "enable"
      ]
    )
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../../lib/modules/generic/assertions.nix
    ../../../../../lib/modules/generic/meta-maintainers.nix
    ../../../config/system-path.nix
    ../../../system/boot/systemd.nix
    ../../system/dbus.nix
    # END generated imports
  ];

  options = {
    services.gnome.localsearch = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to enable LocalSearch, indexing services for TinySPARQL
          search engine and metadata storage system.
        '';
      };
    };
  };

  config = lib.mkIf config.services.gnome.localsearch.enable {
    environment.systemPackages = [ pkgs.localsearch ];

    services.dbus.packages = [ pkgs.localsearch ];

    systemd.packages = [ pkgs.localsearch ];
  };
}
