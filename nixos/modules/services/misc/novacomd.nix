{
  config,
  lib,
  pkgs,
  ...
}:
let

  cfg = config.services.novacomd;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options = {
    services.novacomd = {
      enable = lib.mkEnableOption "Novacom service for connecting to WebOS devices";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.webos.novacom ];

    systemd.services.novacomd = {
      description = "Novacom WebOS daemon";
      wantedBy = [ "multi-user.target" ];

      serviceConfig = {
        ExecStart = "${pkgs.webos.novacomd}/sbin/novacomd";
      };
    };
  };

  meta.maintainers = [ ];
}
