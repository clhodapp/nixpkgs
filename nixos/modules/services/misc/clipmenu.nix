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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
