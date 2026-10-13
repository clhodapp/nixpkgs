{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.picosnitch;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options.services.picosnitch = {
    enable = mkEnableOption "picosnitch daemon";
  };
  config = mkIf cfg.enable {
    environment.systemPackages = [ pkgs.picosnitch ];
    systemd.services.picosnitch = {
      description = "picosnitch";
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        Type = "simple";
        Restart = "always";
        RestartSec = 5;
        ExecStart = "${pkgs.picosnitch}/bin/picosnitch start-no-daemon";
        PIDFile = "/run/picosnitch/picosnitch.pid";
      };
    };
  };
}
