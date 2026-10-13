#
{
  config,
  lib,
  pkgs,
  ...
}:
let

  cfg = config.services.irqbalance;

in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options.services.irqbalance = {

    enable = lib.mkEnableOption "irqbalance daemon";

    package = lib.mkPackageOption pkgs "irqbalance" { };

  };

  config = lib.mkIf cfg.enable {

    environment.systemPackages = [ cfg.package ];

    systemd.services.irqbalance.wantedBy = [ "multi-user.target" ];

    systemd.packages = [ cfg.package ];

  };

}
