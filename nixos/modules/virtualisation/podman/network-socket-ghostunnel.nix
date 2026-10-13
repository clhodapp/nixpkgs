{
  config,
  lib,
  pkg,
  ...
}:
let
  inherit (lib)
    mkOption
    types
    ;

  cfg = config.virtualisation.podman.networkSocket;

in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../services/networking/ghostunnel.nix
    ./network-socket.nix
    # END generated imports
  ];

  options.virtualisation.podman.networkSocket = {
    server = mkOption {
      type = types.enum [ "ghostunnel" ];
    };
  };

  config = lib.mkIf (cfg.enable && cfg.server == "ghostunnel") {

    services.ghostunnel = {
      enable = true;
      servers."podman-socket" = {
        inherit (cfg.tls) cert key cacert;
        listen = "${cfg.listenAddress}:${toString cfg.port}";
        target = "unix:/run/podman/podman.sock";
        allowAll = lib.mkDefault true;
      };
    };
    systemd.services.ghostunnel-server-podman-socket.serviceConfig.SupplementaryGroups = [ "podman" ];

  };

  meta.teams = [ lib.teams.podman ];
  meta.maintainers = [ lib.maintainers.roberth ];
}
