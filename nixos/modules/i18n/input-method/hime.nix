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

  config = lib.mkIf (imcfg.enable && imcfg.type == "hime") {
    i18n.inputMethod.package = pkgs.hime;
    environment.variables = {
      GTK_IM_MODULE = "hime";
      QT_IM_MODULE = "hime";
      XMODIFIERS = "@im=hime";
    };
    services.xserver.displayManager.sessionCommands = "${pkgs.hime}/bin/hime &";
  };
}
