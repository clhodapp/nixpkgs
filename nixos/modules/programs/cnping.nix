{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.cnping;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options = {
    programs.cnping = {
      enable = lib.mkEnableOption "a setcap wrapper for cnping";
    };
  };

  config = lib.mkIf cfg.enable {
    security.wrappers.cnping = {
      owner = "root";
      group = "root";
      source = "${pkgs.cnping}/bin/cnping";
      capabilities = "cap_net_raw+ep";
    };
  };
}
