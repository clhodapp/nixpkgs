{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.hardware.libjaylink;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/users-groups.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.libjaylink = {
    enable = lib.mkEnableOption ''
      udev rules for devices supported by libjaylink.
      Add users to the `jlink` group in order to grant
      them access
    '';
    package = lib.mkPackageOption pkgs "libjaylink" { };
  };

  config = lib.mkIf cfg.enable {
    users.groups.jlink = { };
    services.udev.packages = [ cfg.package ];
  };

  meta.maintainers = with lib.maintainers; [ felixsinger ];
}
