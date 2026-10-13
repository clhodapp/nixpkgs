{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.uptimed;
  stateDir = "/var/lib/uptimed";
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../config/users-groups.nix
    ../../misc/ids.nix
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options = {
    services.uptimed = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Enable `uptimed`, allowing you to track
          your highest uptimes.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {

    environment.systemPackages = [ pkgs.uptimed ];

    users.users.uptimed = {
      description = "Uptimed daemon user";
      home = stateDir;
      uid = config.ids.uids.uptimed;
      group = "uptimed";
    };
    users.groups.uptimed = { };

    systemd.services.uptimed = {
      unitConfig.Documentation = "man:uptimed(8) man:uprecords(1)";
      description = "uptimed service";
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        Restart = "on-failure";
        User = "uptimed";
        Nice = 19;
        IOSchedulingClass = "idle";
        PrivateTmp = "yes";
        PrivateNetwork = "yes";
        NoNewPrivileges = "yes";
        StateDirectory = [ "uptimed" ];
        InaccessibleDirectories = "/home";
        ExecStart = "${pkgs.uptimed}/sbin/uptimed -f -p ${stateDir}/pid";
      };

      preStart = ''
        if ! test -f ${stateDir}/bootid ; then
          ${pkgs.uptimed}/sbin/uptimed -b
        fi
      '';
    };
  };
}
