{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.hdapsd;
  hdapsd = [ pkgs.hdapsd ];
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/kernel.nix
    ../../system/boot/systemd.nix
    ../hardware/udev.nix
    # END generated imports
  ];

  options = {
    services.hdapsd.enable = lib.mkEnableOption ''
      Hard Drive Active Protection System Daemon,
      devices are detected and managed automatically by udev and systemd
    '';
  };

  config = lib.mkIf cfg.enable {
    boot.kernelModules = [ "hdapsd" ];
    services.udev.packages = hdapsd;
    systemd.packages = hdapsd;
  };
}
