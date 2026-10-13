{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.iio-hyprland;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../hardware/sensor/iio.nix
    # END generated imports
  ];

  options = {
    programs.iio-hyprland = {
      enable = lib.mkEnableOption "iio-hyprland and iio-sensor-proxy";
      package = lib.mkPackageOption pkgs "iio-hyprland" { };
    };
  };

  config = lib.mkIf cfg.enable {
    hardware.sensor.iio.enable = lib.mkDefault true;

    environment.systemPackages = [ cfg.package ];
  };
  meta.maintainers = with lib.maintainers; [ yusuf-duran ];
}
