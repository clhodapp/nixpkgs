# Bamf
{
  config,
  lib,
  pkgs,
  ...
}:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../system/boot/systemd.nix
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
