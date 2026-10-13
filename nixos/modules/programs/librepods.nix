{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.librepods;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../config/users-groups.nix
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options = {
    programs.librepods = {
      enable = lib.mkOption {
        default = false;
        type = lib.types.bool;
        description = ''
          Whether to configure system to enable librepods.
          To grant access to a user, it must be part of librepods group:
          `users.users.alice.extraGroups = ["librepods"];`
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [ librepods ];
    users.groups.librepods = { };

    security.wrappers.librepods = {
      source = lib.getExe pkgs.librepods;
      capabilities = "cap_net_admin+ep";
      owner = "root";
      group = "librepods";
      permissions = "u+rx,g+x";
    };
  };

  meta.maintainers = with lib.maintainers; [
    thefossguy
    Cameo007
  ];
}
