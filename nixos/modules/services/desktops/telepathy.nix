# Telepathy daemon.
{
  config,
  lib,
  pkgs,
  ...
}:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../system/dbus.nix
    # END generated imports
  ];

  meta = {
    maintainers = [ ];
  };

  ###### interface

  options = {

    services.telepathy = {

      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to enable Telepathy service, a communications framework
          that enables real-time communication via pluggable protocol backends.
        '';
      };

    };

  };

  ###### implementation

  config = lib.mkIf config.services.telepathy.enable {

    environment.systemPackages = [ pkgs.telepathy-mission-control ];

    services.dbus.packages = [ pkgs.telepathy-mission-control ];
  };

}
