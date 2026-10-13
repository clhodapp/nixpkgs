{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.greenclip;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../system/boot/systemd/user.nix
    # END generated imports
  ];

  options.services.greenclip = {
    enable = lib.mkEnableOption "Greenclip, a clipboard manager";

    package = lib.mkPackageOption pkgs [ "haskellPackages" "greenclip" ] { };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.greenclip = {
      enable = true;
      description = "greenclip daemon";
      wantedBy = [ "graphical-session.target" ];
      after = [ "graphical-session.target" ];
      serviceConfig = {
        ExecStart = "${cfg.package}/bin/greenclip daemon";
        Restart = "always";
      };
    };

    environment.systemPackages = [ cfg.package ];
  };
}
