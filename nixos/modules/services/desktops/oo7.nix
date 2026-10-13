{
  config,
  pkgs,
  lib,
  ...
}:
let
  cfg = config.services.oo7;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../config/xdg/portal.nix
    ../../security/pam.nix
    ../../security/wrappers/default.nix
    ../../system/boot/systemd.nix
    ../../system/boot/systemd/user.nix
    ../system/dbus.nix
    # END generated imports
  ];

  options.services.oo7 = {
    enable = lib.mkEnableOption ''
      oo7, a service providing the Secret Service D-Bus API
    '';
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.oo7 ];
    services.dbus.packages = [ pkgs.oo7-server ];

    systemd = {
      packages = [ pkgs.oo7-server ];
      user.services.oo7-daemon.wantedBy = [ "default.target" ];
    };

    security = {
      pam.services.login.oo7.enable = true;

      wrappers.oo7-daemon = {
        owner = "root";
        group = "root";
        capabilities = "cap_ipc_lock=ep";
        source = "${pkgs.oo7-server}/libexec/oo7-daemon";
      };
    };

    xdg.portal.extraPortals = [ pkgs.oo7-portal ];
  };

  meta = { inherit (pkgs.oo7.meta) maintainers; };
}
