{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.hardware.ubertooth;

  ubertoothPkg = pkgs.ubertooth.override {
    udevGroup = cfg.group;
  };
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../config/users-groups.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.ubertooth = {
    enable = lib.mkEnableOption "Ubertooth software and its udev rules";

    group = lib.mkOption {
      type = lib.types.str;
      default = "ubertooth";
      example = "wheel";
      description = "Group for Ubertooth's udev rules.";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ ubertoothPkg ];

    services.udev.packages = [ ubertoothPkg ];
    users.groups.${cfg.group} = { };
  };
}
