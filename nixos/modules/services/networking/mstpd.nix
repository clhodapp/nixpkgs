{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.mstpd;
in
with lib;
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options.services.mstpd = {

    enable = mkOption {
      default = false;
      type = types.bool;
      description = ''
        Whether to enable the multiple spanning tree protocol daemon.
      '';
    };

  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ pkgs.mstpd ];

    systemd.services.mstpd = {
      description = "Multiple Spanning Tree Protocol Daemon";
      wantedBy = [ "network.target" ];
      unitConfig.ConditionCapability = "CAP_NET_ADMIN";
      serviceConfig = {
        Type = "forking";
        ExecStart = "@${pkgs.mstpd}/bin/mstpd mstpd";
        PIDFile = "/run/mstpd.pid";
      };
    };
  };
}
