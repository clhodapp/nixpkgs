{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.calls;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/system/dbus.nix
    ./dconf.nix
    # END generated imports
  ];

  options = {
    programs.calls = {
      enable = lib.mkEnableOption ''
        GNOME calls: a phone dialer and call handler
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    programs.dconf.enable = true;

    environment.systemPackages = [
      pkgs.calls
    ];

    services.dbus.packages = [
      pkgs.callaudiod
    ];
  };
}
