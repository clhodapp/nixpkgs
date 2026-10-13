{
  lib,
  pkgs,
  config,
  ...
}:
let
  cfg = config.programs.idescriptor;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../config/users-groups.nix
    ../services/hardware/udev.nix
    ../services/hardware/usbmuxd.nix
    # END generated imports
  ];

  options.programs.idescriptor = {
    enable = lib.mkEnableOption "iDescriptor, a cross-platform iDevice management tool";

    package = lib.mkPackageOption pkgs "idescriptor" { };

    users = lib.mkOption {
      type = with lib.types; listOf str;
      default = [ ];
      description = "Users to be added to the idevice group.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    services = {
      udev.packages = [ cfg.package ];

      usbmuxd.enable = true;
    };

    users.groups.idevice = {
      members = cfg.users;
    };
  };

  meta.maintainers = with lib.maintainers; [ amadejkastelic ];
}
