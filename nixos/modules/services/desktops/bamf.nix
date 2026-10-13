# Bamf
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

  meta = {
    maintainers = [ ];
  };

  ###### interface

  options = {
    services.bamf = {
      enable = lib.mkEnableOption "bamf";
    };
  };

  ###### implementation

  config = lib.mkIf config.services.bamf.enable {
    services.dbus.packages = [ pkgs.bamf ];

    systemd.packages = [ pkgs.bamf ];
  };
}
