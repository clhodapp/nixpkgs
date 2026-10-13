{
  config,
  pkgs,
  lib,
  ...
}:

let

  cfg = config.programs._1password;

in
{
  imports = [
    (lib.mkRemovedOptionModule [ "programs" "_1password" "gid" ] ''
      A preallocated GID will be used instead.
    '')
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/assertions.nix
    ../config/system-path.nix
    ../config/users-groups.nix
    ../misc/ids.nix
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options = {
    programs._1password = {
      enable = lib.mkEnableOption "the 1Password CLI tool";

      package = lib.mkPackageOption pkgs "1Password CLI" {
        default = [ "_1password-cli" ];
      };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    users.groups.onepassword-cli.gid = config.ids.gids.onepassword-cli;

    security.wrappers = {
      "op" = {
        source = "${cfg.package}/bin/op";
        owner = "root";
        group = "onepassword-cli";
        setuid = false;
        setgid = true;
      };
    };
  };
}
