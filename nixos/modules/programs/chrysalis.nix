{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.chrysalis;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options = {
    programs.chrysalis = {
      enable = lib.mkEnableOption "Chrysalis";
      package = lib.mkPackageOption pkgs "Chrysalis" { default = "chrysalis"; };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.udev.packages = [ cfg.package ];
  };

  meta.maintainers = with lib.maintainers; [ atalii ];
}
