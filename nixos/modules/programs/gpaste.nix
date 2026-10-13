# GPaste.
{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/desktop-managers/gnome.nix
    ../services/system/dbus.nix
    ../services/x11/desktop-managers/cinnamon.nix
    ../system/boot/systemd.nix
    # END generated imports
  ];

  ###### interface
  options = {
    programs.gpaste = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to enable GPaste, a clipboard manager.
        '';
      };
    };
  };

  ###### implementation
  config = lib.mkIf config.programs.gpaste.enable {
    environment.systemPackages = [ pkgs.gpaste ];
    services.dbus.packages = [ pkgs.gpaste ];
    systemd.packages = [ pkgs.gpaste ];
    # gnome-control-center crashes in Keyboard Shortcuts pane without the GSettings schemas.
    services.desktopManager.gnome.sessionPath = [ pkgs.gpaste ];
    # gpaste-reloaded applet doesn't work without the typelib
    services.xserver.desktopManager.cinnamon.sessionPath = [ pkgs.gpaste ];
  };
}
