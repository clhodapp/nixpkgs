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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
