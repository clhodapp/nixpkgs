{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.systembus-notify;

  inherit (lib) mkEnableOption mkIf;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/systemd.nix
    ../../system/boot/systemd/user.nix
    # END generated imports
  ];

  options.services.systembus-notify = {
    enable = mkEnableOption ''
      System bus notification support

      WARNING: enabling this option (while convenient) should *not* be done on a
      machine where you do not trust the other users as it allows any other
      local user to DoS your session by spamming notifications
    '';
  };

  config = mkIf cfg.enable {
    systemd = {
      packages = with pkgs; [ systembus-notify ];

      user.services.systembus-notify.wantedBy = [ "graphical-session.target" ];
    };
  };
}
