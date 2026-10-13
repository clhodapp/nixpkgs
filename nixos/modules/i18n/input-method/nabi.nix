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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../config/shells-environment.nix
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
