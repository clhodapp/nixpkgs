{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.inputmodule.enable = lib.mkEnableOption "Support for Framework input modules";

  config = lib.mkIf config.hardware.inputmodule.enable {
    environment.systemPackages = [ pkgs.inputmodule-control ];
    services.udev.packages = [ pkgs.inputmodule-control ];
  };
}
