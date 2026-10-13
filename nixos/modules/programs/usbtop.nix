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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
