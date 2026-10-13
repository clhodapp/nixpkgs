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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
