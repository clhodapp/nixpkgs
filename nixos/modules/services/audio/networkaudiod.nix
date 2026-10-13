{
  config,
  lib,
  pkgs,
  ...
}:
let
  name = "networkaudiod";
  cfg = config.services.networkaudiod;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options = {
    services.networkaudiod = {
      enable = lib.mkEnableOption "Networkaudiod (NAA)";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.packages = [ pkgs.networkaudiod ];
    systemd.services.networkaudiod.wantedBy = [ "multi-user.target" ];
  };
}
