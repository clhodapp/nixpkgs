{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.minipro;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../config/users-groups.nix
    # END generated imports
  ];

  options = {
    programs.minipro = {
      enable = lib.mkEnableOption "minipro" // {
        description = ''
          Whether to enable minipro and its udev rules.
          Users of the `plugdev` group can interact with connected MiniPRO chip programmers.
        '';
      };

      package = lib.mkPackageOption pkgs "minipro" { };
    };
  };

  config = lib.mkIf cfg.enable {
    users.groups.plugdev = { };
    environment.systemPackages = [ cfg.package ];
    services.udev.packages = [ cfg.package ];
  };

  meta = {
    maintainers = with lib.maintainers; [ infinidoge ];
  };
}
