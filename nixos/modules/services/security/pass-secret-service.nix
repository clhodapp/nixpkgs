{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.passSecretService;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../system/dbus.nix
    # END generated imports
  ];

  options.services.passSecretService = {
    enable = lib.mkEnableOption "pass secret service";

    package = lib.mkPackageOption pkgs "pass-secret-service" {
      example = "pass-secret-service.override { python3 = pkgs.python315 }";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.packages = [ cfg.package ];
    services.dbus.packages = [ cfg.package ];
  };

  meta.maintainers = with lib.maintainers; [ jluttine ];
}
