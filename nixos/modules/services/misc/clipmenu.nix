{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.clipmenu;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../system/boot/systemd/user.nix
    # END generated imports
  ];

  options.services.clipmenu = {
    enable = lib.mkEnableOption "clipmenu, the clipboard management daemon";

    package = lib.mkPackageOption pkgs "clipmenu" { };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.clipmenu = {
      enable = true;
      description = "Clipboard management daemon";
      wantedBy = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig.ExecStart = "${cfg.package}/bin/clipmenud";
    };

    environment.systemPackages = [ cfg.package ];
  };
}
