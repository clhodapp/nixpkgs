{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.do-agent;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options.services.do-agent = {
    enable = lib.mkEnableOption "do-agent, the DigitalOcean droplet metrics agent";
  };

  config = lib.mkIf cfg.enable {
    systemd.packages = [ pkgs.do-agent ];

    systemd.services.do-agent = {
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = [
          ""
          "${pkgs.do-agent}/bin/do-agent --syslog"
        ];
        DynamicUser = true;
      };
    };
  };
}
