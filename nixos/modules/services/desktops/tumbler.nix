# Tumbler
{
  config,
  pkgs,
  lib,
  ...
}:
let

  cfg = config.services.tumbler;

in

{

  imports = [
    (lib.mkRemovedOptionModule [ "services" "tumbler" "package" ] "")
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/assertions.nix
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/system-path.nix
    ../system/dbus.nix
    # END generated imports
  ];

  meta = {
    teams = [ lib.teams.pantheon ];
  };

  ###### interface

  options = {

    services.tumbler = {

      enable = lib.mkEnableOption "Tumbler, A D-Bus thumbnailer service";

    };

  };

  ###### implementation

  config = lib.mkIf cfg.enable {

    environment.systemPackages = with pkgs; [
      tumbler
    ];

    services.dbus.packages = with pkgs; [
      tumbler
    ];

  };

}
