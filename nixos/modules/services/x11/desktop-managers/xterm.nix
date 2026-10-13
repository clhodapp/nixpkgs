{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let

  cfg = config.services.xserver.desktopManager.xterm;
  xSessionEnabled = config.services.xserver.enable;

in

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../../config/system-path.nix
    ../../../misc/version.nix
    ../xserver.nix
    ./default.nix
    # END generated imports
  ];

  options = {

    services.xserver.desktopManager.xterm.enable = mkOption {
      type = types.bool;
      default = versionOlder config.system.stateVersion "19.09" && xSessionEnabled;
      defaultText = literalExpression ''versionOlder config.system.stateVersion "19.09" && config.services.xserver.enable;'';
      description = "Enable a xterm terminal as a desktop manager.";
    };

  };

  config = mkIf cfg.enable {

    services.xserver.desktopManager.session = singleton {
      name = "xterm";
      start = ''
        ${pkgs.xterm}/bin/xterm -ls &
        waitPID=$!
      '';
    };

    environment.systemPackages = [ pkgs.xterm ];

  };

}
