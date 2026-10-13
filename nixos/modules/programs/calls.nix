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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
