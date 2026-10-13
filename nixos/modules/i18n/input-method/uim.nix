{
  config,
  pkgs,
  lib,
  ...
}:
let
  imcfg = config.i18n.inputMethod;
  cfg = imcfg.uim;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../config/shells-environment.nix
    # END generated imports
  ];

  options = {

    i18n.inputMethod.uim = {
      toolbar = lib.mkOption {
        type = lib.types.enum [
          "gtk"
          "gtk3"
          "gtk-systray"
          "gtk3-systray"
          "qt5"
        ];
        default = "gtk";
        example = "gtk-systray";
        description = ''
          selected UIM toolbar.
        '';
      };
    };

  };

  config = lib.mkIf (imcfg.enable && imcfg.type == "uim") {
    i18n.inputMethod.package = pkgs.uim;

    environment.variables = {
      GTK_IM_MODULE = "uim";
      QT_IM_MODULE = "uim";
      XMODIFIERS = "@im=uim";
    };
    services.xserver.displayManager.sessionCommands = ''
      ${pkgs.uim}/bin/uim-xim &
      ${pkgs.uim}/bin/uim-toolbar-${cfg.toolbar} &
    '';
  };
}
