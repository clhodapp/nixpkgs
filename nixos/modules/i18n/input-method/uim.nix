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
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/shells-environment.nix
    ../../services/x11/display-managers/default.nix
    ./default.nix
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
