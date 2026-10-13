{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.streamcontroller;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/desktop-managers/plasma6.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.programs.streamcontroller = {
    enable = lib.mkEnableOption "StreamController";
    package = lib.mkOption {
      default = pkgs.streamcontroller.override { isKde = config.services.desktopManager.plasma6.enable; };
      defaultText = lib.literalExpression "pkgs.streamcontroller";
      type = lib.types.package;
      description = ''
        The StreamController package to use
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.udev.packages = [ cfg.package ];
  };

  meta.maintainers = with lib.maintainers; [ sifmelcara ];
}
