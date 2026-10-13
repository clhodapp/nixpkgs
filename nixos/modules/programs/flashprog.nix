{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.flashprog;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../hardware/libftdi.nix
    ../hardware/libjaylink.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.programs.flashprog = {
    enable = lib.mkEnableOption ''
      configuring flashprog udev rules and
      installing flashprog as system package
    '';
    package = lib.mkPackageOption pkgs "flashprog" { };
  };

  config = lib.mkIf cfg.enable {
    services.udev.packages = [ cfg.package ];
    environment.systemPackages = [ cfg.package ];
    hardware.libjaylink.enable = true;
    hardware.libftdi.enable = true;
  };

  meta.maintainers = with lib.maintainers; [ felixsinger ];
}
