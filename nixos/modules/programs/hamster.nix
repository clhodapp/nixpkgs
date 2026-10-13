{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/system/dbus.nix
    # END generated imports
  ];

  meta.maintainers = pkgs.hamster.meta.maintainers;

  options.programs.hamster.enable = lib.mkEnableOption "hamster, a time tracking program";

  config = lib.mkIf config.programs.hamster.enable {
    environment.systemPackages = [ pkgs.hamster ];
    services.dbus.packages = [ pkgs.hamster ];
  };
}
