{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.virtualisation.incus.agent;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../services/hardware/udev.nix
    ../system/boot/systemd.nix
    ./incus.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.lxc ];
  };

  options = {
    virtualisation.incus.agent.enable = lib.mkEnableOption "Incus agent";
  };

  config = lib.mkIf cfg.enable {
    services.udev.packages = [ config.virtualisation.incus.package.agent_loader ];
    systemd.packages = [ config.virtualisation.incus.package.agent_loader ];

    systemd.services.incus-agent = {
      enable = true;
      wantedBy = [ "multi-user.target" ];

      path = [
        pkgs.kmod
        pkgs.util-linux

        # allow `incus exec` to find system binaries
        "/run/current-system/sw"
      ];

      # avoid killing nixos-rebuild switch when executed through incus exec
      restartIfChanged = false;
      stopIfChanged = false;
    };
  };
}
