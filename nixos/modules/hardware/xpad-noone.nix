{
  config,
  lib,
  ...
}:

let
  cfg = config.hardware.xpad-noone;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../system/boot/kernel.nix
    ../system/boot/modprobe.nix
    # END generated imports
  ];

  options.hardware.xpad-noone = {
    enable = lib.mkEnableOption "the Xpad driver from the Linux kernel with support for Xbox One controllers removed";
  };

  config = lib.mkIf cfg.enable {
    boot = {
      blacklistedKernelModules = [ "xpad" ];
      extraModulePackages = with config.boot.kernelPackages; [ xpad-noone ];
    };
  };

  meta = {
    maintainers = with lib.maintainers; [ Cryolitia ];
  };
}
