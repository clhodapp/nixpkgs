{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.partition-manager;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/system/dbus.nix
    # END generated imports
  ];

  meta.maintainers = [ lib.maintainers.oxalica ];

  options = {
    programs.partition-manager = {
      enable = lib.mkEnableOption "KDE Partition Manager";

      package = lib.mkPackageOption pkgs [ "kdePackages" "partitionmanager" ] { };
    };
  };

  config = lib.mkIf config.programs.partition-manager.enable {
    services.dbus.packages = [ cfg.package.kpmcore ];
    # `kpmcore` need to be installed to pull in polkit actions.
    environment.systemPackages = [
      cfg.package.kpmcore
      cfg.package
    ];
  };
}
