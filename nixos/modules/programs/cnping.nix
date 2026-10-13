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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
