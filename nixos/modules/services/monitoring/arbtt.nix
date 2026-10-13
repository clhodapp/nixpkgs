{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.arbtt;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/systemd/user.nix
    # END generated imports
  ];

  options = {
    services.arbtt = {
      enable = lib.mkEnableOption "Arbtt statistics capture service";

      package = lib.mkPackageOption pkgs [ "haskellPackages" "arbtt" ] { };

      logFile = lib.mkOption {
        type = lib.types.str;
        default = "%h/.arbtt/capture.log";
        example = "/home/username/.arbtt-capture.log";
        description = ''
          The log file for captured samples.
        '';
      };

      sampleRate = lib.mkOption {
        type = lib.types.int;
        default = 60;
        example = 120;
        description = ''
          The sampling interval in seconds.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.arbtt = {
      description = "arbtt statistics capture service";
      wantedBy = [ "graphical-session.target" ];
      partOf = [ "graphical-session.target" ];

      serviceConfig = {
        Type = "simple";
        ExecStart = "${cfg.package}/bin/arbtt-capture --logfile=${cfg.logFile} --sample-rate=${toString cfg.sampleRate}";
        Restart = "always";
      };
    };
  };

  meta.maintainers = [ ];
}
