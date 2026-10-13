{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.programs.usbtop;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../system/boot/kernel.nix
    # END generated imports
  ];

  options = {
    programs.usbtop.enable = lib.mkEnableOption "usbtop and required kernel module, to show estimated USB bandwidth";
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [
      usbtop
    ];

    boot.kernelModules = [
      "usbmon"
    ];
  };
}
