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
    ../services/desktops/system-config-printer.nix
    # END generated imports
  ];

  ###### interface

  options = {

    programs.system-config-printer = {

      enable = lib.mkEnableOption "system-config-printer, a Graphical user interface for CUPS administration";

    };

  };

  ###### implementation

  config = lib.mkIf config.programs.system-config-printer.enable {

    environment.systemPackages = [
      pkgs.system-config-printer
    ];

    services.system-config-printer.enable = true;

  };

}
