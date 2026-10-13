{
  config,
  lib,
  pkgs,
  ...
}:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../services/hardware/udev.nix
    ../../services/system/dbus.nix
    ../../system/boot/kernel.nix
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  ###### interface

  options = {
    hardware.sensor.iio = {
      enable = lib.mkOption {
        description = ''
          Enable this option to support IIO sensors with iio-sensor-proxy.

          IIO sensors are used for orientation and ambient light
          sensors on some mobile devices.
        '';
        type = lib.types.bool;
        default = false;
      };

      package = lib.mkPackageOption pkgs "iio-sensor-proxy" { };
    };
  };

  ###### implementation

  config = lib.mkIf config.hardware.sensor.iio.enable {

    boot.initrd.availableKernelModules = [ "hid-sensor-hub" ];

    environment.systemPackages = [ config.hardware.sensor.iio.package ];

    services.dbus.packages = [ config.hardware.sensor.iio.package ];
    services.udev.packages = [ config.hardware.sensor.iio.package ];
    systemd.packages = [ config.hardware.sensor.iio.package ];
  };
}
