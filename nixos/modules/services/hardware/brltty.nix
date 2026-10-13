{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.brltty;

  targets = [
    "default.target"
    "multi-user.target"
    "rescue.target"
    "emergency.target"
  ];

  genApiKey = pkgs.writers.writeDash "generate-brlapi-key" ''
    if ! test -f /etc/brlapi.key; then
      echo -n generating brlapi key...
      ${pkgs.brltty}/bin/brltty-genkey -f /etc/brlapi.key
      echo done
    fi
  '';

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../config/users-groups.nix
    ../../system/boot/systemd.nix
    ../../system/boot/systemd/tmpfiles.nix
    ./udev.nix
    # END generated imports
  ];

  options = {

    services.brltty.enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = "Whether to enable the BRLTTY daemon.";
    };

  };

  config = lib.mkIf cfg.enable {
    users.users.brltty = {
      description = "BRLTTY daemon user";
      group = "brltty";
      isSystemUser = true;
    };
    users.groups = {
      brltty = { };
      brlapi = { };
    };

    systemd.services."brltty@".serviceConfig = {
      ExecStartPre = "!${genApiKey}";
    };

    # Install all upstream-provided files
    systemd.packages = [ pkgs.brltty ];
    systemd.tmpfiles.packages = [ pkgs.brltty ];
    services.udev.packages = [ pkgs.brltty ];
    environment.systemPackages = [ pkgs.brltty ];

    # Add missing WantedBys (see issue #81138)
    systemd.paths.brltty.wantedBy = targets;
    systemd.paths."brltty@".wantedBy = targets;
  };

}
