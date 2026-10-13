{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.hardware.rtl-sdr;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../config/users-groups.nix
    ../services/hardware/udev.nix
    ../system/boot/modprobe.nix
    # END generated imports
  ];

  options.hardware.rtl-sdr = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Enables rtl-sdr udev rules, ensures 'plugdev' group exists, and blacklists DVB kernel modules.
        This is a prerequisite to using devices supported by rtl-sdr without being root, since rtl-sdr USB descriptors will be owned by plugdev through udev.
      '';
    };
    package = lib.mkPackageOption pkgs "rtl-sdr" { };
  };

  config = lib.mkIf cfg.enable {
    boot.blacklistedKernelModules = [
      "dvb_usb_rtl28xxu"
      "e4000"
      "rtl2832"
    ];
    services.udev.packages = [ cfg.package ];
    environment.systemPackages = [ cfg.package ];
    users.groups.plugdev = { };
  };

  meta.maintainers = with lib.maintainers; [ pandapip1 ];
}
