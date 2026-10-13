{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.hardware.spacenavd;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options = {
    hardware.spacenavd = {
      enable = lib.mkEnableOption "spacenavd to support 3DConnexion devices";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd = {
      packages = [ pkgs.spacenavd ];
      services.spacenavd = {
        enable = true;
        wantedBy = [ "graphical.target" ];
      };
    };
  };
}
