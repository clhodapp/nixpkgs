{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.hardware.cpu.intel.npu;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../services/hardware/udev.nix
    ../graphics.nix
    # END generated imports
  ];

  options = {
    hardware.cpu.intel.npu = {
      enable = lib.mkEnableOption "Intel NPU support";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [
      pkgs.intel-npu-driver.validation
      pkgs.level-zero
    ];

    hardware = {
      firmware = [ pkgs.intel-npu-driver.firmware ];
      graphics = {
        enable = true;
        extraPackages = [ pkgs.intel-npu-driver ];
      };
    };
  };
}
