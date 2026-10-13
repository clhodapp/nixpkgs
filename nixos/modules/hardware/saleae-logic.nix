{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.hardware.saleae-logic;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.saleae-logic = {
    enable = lib.mkEnableOption "udev rules for Saleae Logic devices";

    package = lib.mkOption {
      type = lib.types.package;
      default = pkgs.saleae-logic-2;
      defaultText = lib.literalExpression "pkgs.saleae-logic-2";
      description = ''
        Saleae Logic package to use.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    services.udev.packages = [ cfg.package ];
  };

  meta.maintainers = with lib.maintainers; [ chivay ];
}
