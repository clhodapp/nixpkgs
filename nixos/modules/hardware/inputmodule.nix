{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.inputmodule.enable = lib.mkEnableOption "Support for Framework input modules";

  config = lib.mkIf config.hardware.inputmodule.enable {
    environment.systemPackages = [ pkgs.inputmodule-control ];
    services.udev.packages = [ pkgs.inputmodule-control ];
  };
}
