{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.services.linyaps;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/fonts/fontdir.nix
    ../../config/shells-environment.nix
    ../../config/system-path.nix
    ../../config/users-groups.nix
    ../../security/polkit.nix
    ../../system/boot/systemd.nix
    ../../system/boot/systemd/tmpfiles.nix
    ../system/dbus.nix
    # END generated imports
  ];

  meta = {
    maintainers = pkgs.linyaps.meta.maintainers;
  };

  ###### interface
  options = {
    services.linyaps = {
      enable = lib.mkEnableOption "linyaps, a cross-distribution package manager with sandboxed apps and shared runtime";

      package = lib.mkPackageOption pkgs "linyaps" { };

      boxPackage = lib.mkPackageOption pkgs "linyaps-box" { };

      webStoreInstallerPackage = lib.mkPackageOption pkgs "linyaps-web-store-installer" { };
    };
  };

  ###### implementation
  config = lib.mkIf cfg.enable {

    environment = {
      profiles = [ "/var/lib/linglong/entries" ];
      systemPackages = [
        cfg.package
        cfg.boxPackage
        cfg.webStoreInstallerPackage
      ];
    };

    security.polkit.enable = true;

    fonts.fontDir.enable = true;

    services.dbus.packages = [ cfg.package ];

    systemd = {
      packages = [ cfg.package ];
      tmpfiles.packages = [ cfg.package ];
    };

    # Create system user and group for linyaps/linglong
    users = {
      groups.deepin-linglong = { };
      users.deepin-linglong = {
        group = "deepin-linglong";
        isSystemUser = true;
        description = "Linyaps/Linglong system helper";
      };
    };
  };
}
