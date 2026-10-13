{
  config,
  lib,
  pkgs,
  ...
}:
let

  cfg = config.services.fstrim;

in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options = {

    services.fstrim = {
      enable = (
        lib.mkEnableOption "periodic SSD TRIM of mounted partitions in background"
        // {
          default = true;
        }
      );

      interval = lib.mkOption {
        type = lib.types.str;
        default = "weekly";
        description = ''
          How often we run fstrim. For most desktop and server systems
          a sufficient trimming frequency is once a week.

          The format is described in
          {manpage}`systemd.time(7)`.
        '';
      };
    };

  };

  config = lib.mkIf cfg.enable {

    systemd.packages = [ pkgs.util-linux ];

    systemd.timers.fstrim = {
      timerConfig = {
        OnCalendar = [
          ""
          cfg.interval
        ];
      };
      wantedBy = [ "timers.target" ];
    };

  };

  meta.maintainers = [ ];
}
