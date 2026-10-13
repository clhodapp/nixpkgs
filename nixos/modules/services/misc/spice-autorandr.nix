{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.services.spice-autorandr;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../system/boot/systemd/user.nix
    # END generated imports
  ];

  options = {
    services.spice-autorandr = {
      enable = lib.mkEnableOption "spice-autorandr service that will automatically resize display to match SPICE client window size";
      package = lib.mkPackageOption pkgs "spice-autorandr" { };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    systemd.user.services.spice-autorandr = {
      wantedBy = [ "default.target" ];
      after = [ "spice-vdagentd.service" ];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/spice-autorandr";
        Restart = "on-failure";
      };
    };
  };
}
