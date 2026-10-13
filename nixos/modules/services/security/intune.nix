{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.intune;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../config/users-groups.nix
    ../../system/boot/systemd.nix
    ../../system/boot/systemd/tmpfiles.nix
    ../system/dbus.nix
    # END generated imports
  ];

  options.services.intune = {
    enable = lib.mkEnableOption "Microsoft Intune";
  };

  config = lib.mkIf cfg.enable {
    users.users.microsoft-identity-broker = {
      group = "microsoft-identity-broker";
      isSystemUser = true;
    };

    users.groups.microsoft-identity-broker = { };
    environment.systemPackages = [
      pkgs.microsoft-identity-broker
      pkgs.intune-portal
    ];
    systemd.packages = [
      pkgs.microsoft-identity-broker
      pkgs.intune-portal
    ];

    systemd.tmpfiles.packages = [ pkgs.intune-portal ];
    services.dbus.packages = [ pkgs.microsoft-identity-broker ];
  };

  meta = {
    maintainers = with lib.maintainers; [ rhysmdnz ];
  };
}
