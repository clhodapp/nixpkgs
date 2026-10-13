{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.hardware.glasgow;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../config/users-groups.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.glasgow = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Enables Glasgow udev rules and ensures 'plugdev' group exists.
        This is a prerequisite to using Glasgow without being root.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.glasgow ];
    services.udev.packages = [ pkgs.glasgow ];
    users.groups.plugdev = { };
  };
}
