{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.ns-usbloader;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options = {
    programs.ns-usbloader = {
      enable = lib.mkEnableOption "ns-usbloader application with udev rules applied";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.ns-usbloader ];
    services.udev.packages = [ pkgs.ns-usbloader ];
  };

  meta.maintainers = pkgs.ns-usbloader.meta.maintainers;
}
