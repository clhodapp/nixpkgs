{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.desktopManager.kodi;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../config/system-path.nix
    ../../networking/firewalld
    ./default.nix
    # END generated imports
  ];

  options = {
    services.xserver.desktopManager.kodi = {
      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Enable the kodi multimedia center.";
      };

      package = mkPackageOption pkgs "kodi" {
        example = "kodi.withPackages (p: with p; [ jellyfin pvr-iptvsimple vfs-sftp ])";
      };
    };
  };

  config = mkIf cfg.enable {
    services.xserver.desktopManager.session = [
      {
        name = "kodi";
        start = ''
          LIRC_SOCKET_PATH=/run/lirc/lircd ${cfg.package}/bin/kodi --standalone &
          waitPID=$!
        '';
      }
    ];

    environment.systemPackages = [ cfg.package ];

    services.firewalld.packages = [ cfg.package ];
  };
}
