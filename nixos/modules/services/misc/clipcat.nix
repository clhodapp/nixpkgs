{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.clipcat;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../system/boot/systemd/user.nix
    # END generated imports
  ];

  options.services.clipcat = {
    enable = lib.mkEnableOption "Clipcat clipboard daemon";

    package = lib.mkPackageOption pkgs "clipcat" { };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.clipcat = {
      enable = true;
      description = "clipcat daemon";
      wantedBy = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig.ExecStart = "${cfg.package}/bin/clipcatd --no-daemon";
    };

    environment.systemPackages = [ cfg.package ];
  };
}
