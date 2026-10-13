{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.sysdig;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../system/boot/kernel.nix
    # END generated imports
  ];

  options.programs.sysdig.enable = lib.mkEnableOption "sysdig, a tracing tool";

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.sysdig ];
    boot.extraModulePackages = [ config.boot.kernelPackages.sysdig ];
  };
}
