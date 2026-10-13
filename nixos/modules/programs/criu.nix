{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.criu;
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
    programs.criu = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Install {command}`criu` along with necessary kernel options.
        '';
      };
    };
  };
  config = lib.mkIf cfg.enable {
    system.requiredKernelConfig = with config.lib.kernelConfig; [
      (isYes "CHECKPOINT_RESTORE")
    ];
    boot.kernel.features.criu = true;
    environment.systemPackages = [ pkgs.criu ];
  };

}
