{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../system/dbus.nix
    # END generated imports
  ];

  options = {
    services.sysprof = {
      enable = lib.mkEnableOption "sysprof profiling daemon";
    };
  };

  config = lib.mkIf config.services.sysprof.enable {
    environment.systemPackages = [ pkgs.sysprof ];

    services.dbus.packages = [ pkgs.sysprof ];

    systemd.packages = [ pkgs.sysprof ];
  };

  meta.maintainers = pkgs.sysprof.meta.maintainers;
}
