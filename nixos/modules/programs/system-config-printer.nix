{
  config,
  pkgs,
  lib,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
