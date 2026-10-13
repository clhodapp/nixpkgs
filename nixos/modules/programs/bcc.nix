{
  config,
  pkgs,
  lib,
  ...
}:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../system/boot/kernel.nix
    # END generated imports
  ];

  options.programs.bcc.enable = lib.mkEnableOption "bcc, tools for BPF-based Linux IO analysis, networking, monitoring, and more";

  config = lib.mkIf config.programs.bcc.enable {
    environment.systemPackages = [ pkgs.bcc ];
    boot.extraModulePackages = [ pkgs.bcc ];
  };
}
