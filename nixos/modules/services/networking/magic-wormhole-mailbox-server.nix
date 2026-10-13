{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.magic-wormhole-mailbox-server;
  # keep semicolon in dataDir for backward compatibility
  dataDir = "/var/lib/magic-wormhole-mailbox-server;";
  python = pkgs.python3.withPackages (
    py: with py; [
      magic-wormhole-mailbox-server
      twisted
    ]
  );
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options.services.magic-wormhole-mailbox-server = {
    enable = lib.mkEnableOption "Magic Wormhole Mailbox Server";
  };

  config = lib.mkIf cfg.enable {
    systemd.services.magic-wormhole-mailbox-server = {
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      serviceConfig = {
        DynamicUser = true;
        ExecStart = "${python}/bin/twistd --nodaemon wormhole-mailbox";
        WorkingDirectory = dataDir;
        StateDirectory = baseNameOf dataDir;
      };
    };
  };

  meta.maintainers = [ lib.maintainers.mjoerg ];
}
