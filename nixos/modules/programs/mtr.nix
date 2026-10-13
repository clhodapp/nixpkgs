{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.mtr;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options = {
    programs.mtr = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to add mtr to the global environment and configure a
          setcap wrapper for it.
        '';
      };

      package = lib.mkPackageOption pkgs "mtr" { };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    security.wrappers.mtr-packet = {
      owner = "root";
      group = "root";
      capabilities = "cap_net_raw+p";
      source = "${cfg.package}/bin/mtr-packet";
    };
  };
}
