{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.digitalbitbox;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../hardware/digitalbitbox.nix
    ../../misc/meta.nix
    # END generated imports
  ];

  options.programs.digitalbitbox = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Installs the Digital Bitbox application and enables the complementary hardware module.
      '';
    };

    package = lib.mkPackageOption pkgs "digitalbitbox" {
      extraDescription = ''
        This can be used to install a package with udev rules that differ from the defaults.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    hardware.digitalbitbox = {
      enable = true;
      package = cfg.package;
    };
  };

  meta = {
    doc = ./default.md;
  };
}
