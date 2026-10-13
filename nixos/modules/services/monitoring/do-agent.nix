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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
