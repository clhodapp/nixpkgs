{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.flashrom;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.programs.flashrom = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Installs flashrom and configures udev rules for programmers
        used by flashrom. Grants access to users in the "flashrom"
        group.
      '';
    };
    package = lib.mkPackageOption pkgs "flashrom" { };
  };

  config = lib.mkIf cfg.enable {
    services.udev.packages = [ cfg.package ];
    environment.systemPackages = [ cfg.package ];
  };
}
