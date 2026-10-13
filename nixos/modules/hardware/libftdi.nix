{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.hardware.libftdi;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/users-groups.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.libftdi = {
    enable = lib.mkEnableOption "udev rules for devices supported by libftdi";
    package = lib.mkPackageOption pkgs "libftdi1" { };
  };

  config = lib.mkIf cfg.enable {
    users.groups.ftdi = { };
    services.udev.packages = [ cfg.package ];
  };

  meta.maintainers = with lib.maintainers; [ felixsinger ];
}
