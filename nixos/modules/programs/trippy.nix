{
  lib,
  config,
  pkgs,
  ...
}:

let
  cfg = config.programs.trippy;
in

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options = {
    programs.trippy = {
      enable = lib.mkEnableOption "trippy, a network diagnostic tool";
    };
  };

  config = lib.mkIf cfg.enable {
    security.wrappers.trip = {
      owner = "root";
      group = "root";
      capabilities = "cap_net_raw+p";
      source = lib.getExe pkgs.trippy;
    };
  };

  meta.maintainers = [ ];
}
