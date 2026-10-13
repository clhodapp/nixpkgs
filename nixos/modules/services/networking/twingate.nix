{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.twingate;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./firewall.nix
    # END generated imports
  ];

  options.services.twingate = {
    enable = lib.mkEnableOption "Twingate Client daemon";
    package = lib.mkPackageOption pkgs "twingate" { };
  };

  config = lib.mkIf cfg.enable {
    systemd.packages = [ cfg.package ];
    systemd.services.twingate = {
      serviceConfig.ExecStartPre = "${lib.getExe' pkgs.coreutils "cp"} -r --update=none ${cfg.package}/etc/twingate/. /etc/twingate/";
      wantedBy = [ "multi-user.target" ];
    };

    networking.firewall.checkReversePath = lib.mkDefault "loose";
    services.resolved.enable = lib.mkIf (!config.networking.networkmanager.enable) true;

    environment.systemPackages = [ cfg.package ]; # For the CLI.
  };
}
