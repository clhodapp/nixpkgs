{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.hardware.brillo;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options = {
    hardware.brillo = {
      enable = lib.mkEnableOption ''
        brillo in userspace.
        This will allow brightness control from users in the video group
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    services.udev.packages = [ pkgs.brillo ];
    environment.systemPackages = [ pkgs.brillo ];
  };
}
