{ config, lib, ... }:
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../system-path.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.freedesktop ];
  };

  options = {
    xdg.menus.enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = ''
        Whether to install files to support the
        [XDG Desktop Menu specification](https://specifications.freedesktop.org/menu-spec/latest).
      '';
    };
  };

  config = lib.mkIf config.xdg.menus.enable {
    environment.pathsToLink = [
      "/share/applications"
      "/share/desktop-directories"
      "/etc/xdg/menus"
      "/etc/xdg/menus/applications-merged"
    ];
  };

}
