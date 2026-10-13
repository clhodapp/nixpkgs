{
  config,
  pkgs,
  lib,
  ...
}:
let
  imcfg = config.i18n.inputMethod;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/shells-environment.nix
    ../../services/x11/display-managers/default.nix
    ./default.nix
    # END generated imports
  ];

  config = lib.mkIf (imcfg.enable && imcfg.type == "nabi") {
    i18n.inputMethod.package = pkgs.nabi;

    environment.variables = {
      GTK_IM_MODULE = "nabi";
      QT_IM_MODULE = "nabi";
      XMODIFIERS = "@im=nabi";
    };

    services.xserver.displayManager.sessionCommands = "${pkgs.nabi}/bin/nabi &";
  };
}
