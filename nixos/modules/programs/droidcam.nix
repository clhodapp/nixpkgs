{
  lib,
  pkgs,
  config,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../system/boot/kernel.nix
    # END generated imports
  ];

  options.programs.droidcam = {
    enable = lib.mkEnableOption "DroidCam client";
  };

  config = lib.mkIf config.programs.droidcam.enable {
    environment.systemPackages = [ pkgs.droidcam ];

    boot.extraModulePackages = [ config.boot.kernelPackages.v4l2loopback ];
    boot.kernelModules = [
      "v4l2loopback"
      "snd-aloop"
    ];
  };
}
