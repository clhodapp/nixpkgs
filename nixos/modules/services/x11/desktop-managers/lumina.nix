{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let

  xcfg = config.services.xserver;
  cfg = xcfg.desktopManager.lumina;

in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../../lib/modules/generic/meta-maintainers.nix
    ../../../config/system-path.nix
    ../../display-managers/default.nix
    # END generated imports
  ];

  meta = {
    teams = [ teams.lumina ];
  };

  options = {

    services.xserver.desktopManager.lumina.enable = mkOption {
      type = types.bool;
      default = false;
      description = "Enable the Lumina desktop manager";
    };

  };

  config = mkIf cfg.enable {

    services.displayManager.sessionPackages = [
      pkgs.lumina.lumina
    ];

    environment.systemPackages = pkgs.lumina.preRequisitePackages ++ pkgs.lumina.corePackages;

    # Link some extra directories in /run/current-system/software/share
    environment.pathsToLink = [
      "/share/lumina"
      # FIXME: modules should link subdirs of `/share` rather than relying on this
      "/share"
    ];

  };
}
