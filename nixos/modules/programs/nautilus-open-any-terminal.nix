{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.nautilus-open-any-terminal;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-environment.nix
    ../config/system-path.nix
    ../services/desktop-managers/gnome.nix
    ./dconf.nix
    # END generated imports
  ];

  options.programs.nautilus-open-any-terminal = {
    enable = lib.mkEnableOption "nautilus-open-any-terminal";

    terminal = lib.mkOption {
      type = with lib.types; nullOr str;
      default = null;
      description = ''
        The terminal emulator to add to context-entry of nautilus. Supported terminal
        emulators are listed in <https://github.com/Stunkymonkey/nautilus-open-any-terminal#supported-terminal-emulators>.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [
      nautilus-python
      nautilus-open-any-terminal
    ];

    environment.sessionVariables = lib.mkIf (!config.services.desktopManager.gnome.enable) {
      NAUTILUS_4_EXTENSION_DIR = "${pkgs.nautilus-python}/lib/nautilus/extensions-4";
    };

    environment.pathsToLink = [
      "/share/nautilus-python/extensions"
    ];

    programs.dconf = lib.optionalAttrs (cfg.terminal != null) {
      enable = true;
      profiles.user.databases = [
        {
          settings."com/github/stunkymonkey/nautilus-open-any-terminal".terminal = cfg.terminal;
          lockAll = true;
        }
      ];
    };
  };
  meta = {
    maintainers = with lib.maintainers; [
      stunkymonkey
      linsui
    ];
  };
}
