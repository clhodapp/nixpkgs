{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.rosec;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../config/xdg/portal.nix
    ../../security/pam.nix
    ../../system/boot/systemd.nix
    ../system/dbus.nix
    # END generated imports
  ];

  options.services.rosec = {
    enable = lib.mkEnableOption "rosec, a secrets daemon implementing the freedesktop.org Secret Service API";

    package = lib.mkPackageOption pkgs "rosec" { };

    pam = {
      enable = lib.mkEnableOption "PAM integration to automatically unlock the rosec vault on login";

      services = lib.mkOption {
        type = with lib.types; listOf str;
        default = [ "login" ];
        example = [
          "login"
          "greetd"
        ];
        description = ''
          List of PAM services for which to enable rosec automatic unlock.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    services.dbus.packages = [ cfg.package ];

    systemd.packages = [ cfg.package ];

    xdg.portal.extraPortals = [ cfg.package ];

    security.pam.services = lib.mkIf cfg.pam.enable (
      lib.genAttrs cfg.pam.services (_: {
        rosec.enable = true;
      })
    );
  };

  meta.maintainers = with lib.maintainers; [ mikilio ];
}
