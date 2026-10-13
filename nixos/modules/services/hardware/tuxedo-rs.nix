{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.hardware.tuxedo-rs;

in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../hardware/tuxedo-drivers.nix
    # END generated imports
  ];

  options = {
    hardware.tuxedo-rs = {
      enable = lib.mkEnableOption "Rust utilities for interacting with hardware from TUXEDO Computers";

      tailor-gui.enable = lib.mkEnableOption "tailor-gui, an alternative to TUXEDO Control Center, written in Rust";
    };
  };

  config = lib.mkIf cfg.enable (
    lib.mkMerge [
      {
        hardware.tuxedo-drivers.enable = true;

        systemd = {
          services.tailord = {
            enable = true;
            description = "Tuxedo Tailor hardware control service";
            after = [ "systemd-logind.service" ];
            wantedBy = [ "multi-user.target" ];

            serviceConfig = {
              Type = "dbus";
              BusName = "com.tux.Tailor";
              ExecStart = "${pkgs.tuxedo-rs}/bin/tailord";
              Environment = "RUST_BACKTRACE=1";
              Restart = "on-failure";
            };
          };
        };

        services.dbus.packages = [ pkgs.tuxedo-rs ];

        environment.systemPackages = [ pkgs.tuxedo-rs ];
      }
      (lib.mkIf cfg.tailor-gui.enable {
        environment.systemPackages = [ pkgs.tailor-gui ];
      })
    ]
  );

  meta.maintainers = with lib.maintainers; [
    xaverdh
  ];
}
