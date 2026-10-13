{ config, lib, ... }:

let
  cfg = config.programs.systemtap;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../misc/lib.nix
    ../system/boot/kernel.nix
    # END generated imports
  ];

  options = {
    programs.systemtap = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Install {command}`systemtap` along with necessary kernel options.
        '';
      };
    };
  };
  config = lib.mkIf cfg.enable {
    system.requiredKernelConfig = with config.lib.kernelConfig; [
      (isYes "DEBUG")
    ];
    boot.kernel.features.debug = true;
    environment.systemPackages = [
      config.boot.kernelPackages.systemtap
    ];
  };

}
