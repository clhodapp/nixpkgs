{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
