{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.pacemaker;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../../lib/modules/generic/assertions.nix
    ../../../config/system-path.nix
    ../../../config/users-groups.nix
    ../../../system/boot/systemd.nix
    ../../../system/boot/systemd/tmpfiles.nix
    ../corosync/default.nix
    # END generated imports
  ];

  # interface
  options.services.pacemaker = {
    enable = lib.mkEnableOption "pacemaker";

    package = lib.mkPackageOption pkgs "pacemaker" { };
  };

  # implementation
  config = lib.mkIf cfg.enable {
    assertions = [
      {
        assertion = config.services.corosync.enable;
        message = ''
          Enabling services.pacemaker requires a services.corosync configuration.
        '';
      }
    ];

    environment.systemPackages = [ cfg.package ];

    # required by pacemaker
    users.users.hacluster = {
      isSystemUser = true;
      group = "pacemaker";
      home = "/var/lib/pacemaker";
    };
    users.groups.pacemaker = { };

    systemd.tmpfiles.rules = [
      "d /var/log/pacemaker 0700 hacluster pacemaker -"
    ];

    systemd.packages = [ cfg.package ];
    systemd.services.pacemaker = {
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        StateDirectory = "pacemaker";
        StateDirectoryMode = "0700";
      };
    };
  };
}
