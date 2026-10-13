{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../system/boot/systemd.nix
    ../system/dbus.nix
    # END generated imports
  ];

  options.services.mozillavpn.enable = lib.mkEnableOption "Mozilla VPN daemon";

  config = lib.mkIf config.services.mozillavpn.enable {
    environment.systemPackages = [ pkgs.mozillavpn ];
    services.dbus.packages = [ pkgs.mozillavpn ];
    systemd.packages = [ pkgs.mozillavpn ];
  };

  meta.maintainers = with lib.maintainers; [ andersk ];
}
