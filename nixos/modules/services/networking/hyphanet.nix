{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.hyphanet;
  useNewNames = lib.versionAtLeast config.system.stateVersion "26.11";
  userName = if useNewNames then "hyphanet" else "freenet";
  varDir = "/var/lib/${userName}";
in
{
  options = {
    services.hyphanet = {
      enable = lib.mkEnableOption "Hyphanet daemon";

      nice = lib.mkOption {
        type = lib.types.ints.between (-20) 19;
        default = 10;
        description = "Set the nice level for the Hyphanet daemon";
      };
    };
  };

  imports = [
    (lib.mkRenamedOptionModule [ "services" "freenet" ] [ "services" "hyphanet" ])
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/assertions.nix
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/users-groups.nix
    ../../misc/version.nix
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  config = lib.mkIf cfg.enable {
    systemd.services.hyphanet = {
      description = "Hyphanet daemon";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = lib.getExe pkgs.hyphanet;
        User = userName;
        UMask = "0007";
        WorkingDirectory = varDir;
        Nice = cfg.nice;
      };
    };

    users.users.${userName} = {
      group = userName;
      description = "Hyphanet daemon user";
      home = varDir;
      createHome = true;
    };

    users.groups.${userName} = { };
  };

  meta.maintainers = with lib.maintainers; [ nagy ];
}
