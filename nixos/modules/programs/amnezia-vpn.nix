{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.amnezia-vpn;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/system/dbus.nix
    ../system/boot/resolved.nix
    ../system/boot/systemd.nix
    # END generated imports
  ];

  options.programs.amnezia-vpn = {
    enable = lib.mkEnableOption "The AmneziaVPN client";
    package = lib.mkPackageOption pkgs "amnezia-vpn" { };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    services.dbus.packages = [ cfg.package ];
    services.resolved.enable = true;

    systemd = {
      packages = [ cfg.package ];
      services."AmneziaVPN" = {
        wantedBy = [ "multi-user.target" ];
        path = with pkgs; [
          gawk
          iptables
          procps
          iproute2
          sudo
        ];
      };
    };
  };

  meta.maintainers = with lib.maintainers; [ sund3RRR ];
}
