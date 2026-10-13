{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.traceroute;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options = {
    programs.traceroute = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to configure a setcap wrapper for traceroute.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    security.wrappers.traceroute = {
      owner = "root";
      group = "root";
      capabilities = "cap_net_raw+p";
      source = lib.getExe pkgs.traceroute;
    };
  };
}
