{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.logkeys;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options.services.logkeys = {
    enable = lib.mkEnableOption "logkeys, a keylogger service";

    device = lib.mkOption {
      description = "Use the given device as keyboard input event device instead of /dev/input/eventX default.";
      default = null;
      type = lib.types.nullOr lib.types.str;
      example = "/dev/input/event15";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.services.logkeys = {
      description = "LogKeys Keylogger Daemon";
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        ExecStart = "${pkgs.logkeys}/bin/logkeys -s${
          lib.optionalString (cfg.device != null) " -d ${cfg.device}"
        }";
        ExecStop = "${pkgs.logkeys}/bin/logkeys -k";
        Type = "forking";
      };
    };
  };
}
