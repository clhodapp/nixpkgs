{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.spice-vdagentd;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options = {
    services.spice-vdagentd = {
      enable = lib.mkEnableOption "Spice guest vdagent daemon";
    };
  };

  config = lib.mkIf cfg.enable {

    environment.systemPackages = [ pkgs.spice-vdagent ];

    systemd.services.spice-vdagentd = {
      description = "spice-vdagent daemon";
      wantedBy = [ "graphical.target" ];
      serviceConfig = {
        Type = "forking";
        ExecStart = "${pkgs.spice-vdagent}/bin/spice-vdagentd";
        RuntimeDirectory = "spice-vdagentd";
      };
    };
  };
}
