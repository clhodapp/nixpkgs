{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.ratbagd;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/system-path.nix
    ../../system/boot/systemd.nix
    ../system/dbus.nix
    # END generated imports
  ];

  ###### interface

  options = {
    services.ratbagd = {
      enable = lib.mkEnableOption "ratbagd for configuring gaming mice";

      package = lib.mkPackageOption pkgs "libratbag" { };
    };
  };

  ###### implementation

  config = lib.mkIf cfg.enable {
    # Give users access to the "ratbagctl" tool
    environment.systemPackages = [ cfg.package ];

    services.dbus.packages = [ cfg.package ];

    systemd.packages = [ cfg.package ];
  };
}
