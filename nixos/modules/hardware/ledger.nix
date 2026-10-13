{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.hardware.ledger;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.ledger.enable = lib.mkEnableOption "udev rules for Ledger devices";

  config = lib.mkIf cfg.enable {
    services.udev.packages = [ pkgs.ledger-udev-rules ];
  };
}
