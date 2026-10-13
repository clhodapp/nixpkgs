{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.dmrconfig;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  meta.maintainers = [ ];

  ###### interface
  options = {
    programs.dmrconfig = {
      enable = lib.mkOption {
        default = false;
        type = lib.types.bool;
        description = ''
          Whether to configure system to enable use of dmrconfig. This
          enables the required udev rules and installs the program.
        '';
        relatedPackages = [ "dmrconfig" ];
      };

      package = lib.mkPackageOption pkgs "dmrconfig" { };
    };
  };

  ###### implementation
  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.udev.packages = [ cfg.package ];
  };
}
