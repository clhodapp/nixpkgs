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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../hardware/libftdi.nix
    ../hardware/libjaylink.nix
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
