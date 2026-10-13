# Seahorse.

{
  config,
  pkgs,
  lib,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/system/dbus.nix
    ./ssh.nix
    # END generated imports
  ];

  ###### interface

  options = {

    programs.seahorse = {

      enable = lib.mkEnableOption "Seahorse, a GNOME application for managing encryption keys and passwords in the GNOME Keyring";

    };

  };

  ###### implementation

  config = lib.mkIf config.programs.seahorse.enable {

    programs.ssh.askPassword = lib.mkDefault "${pkgs.seahorse}/libexec/seahorse/ssh-askpass";

    environment.systemPackages = [
      pkgs.seahorse
    ];

    services.dbus.packages = [
      pkgs.seahorse
    ];

  };

}
