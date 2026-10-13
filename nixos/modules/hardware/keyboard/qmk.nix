{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.hardware.keyboard.qmk;
  inherit (lib) mkEnableOption mkIf;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../config/users-groups.nix
    # END generated imports
  ];

  options.hardware.keyboard.qmk = {
    enable = mkEnableOption "non-root access to the firmware of QMK keyboards";
    keychronSupport = mkEnableOption "udev rules for keychron QMK based keyboards";
  };

  config = mkIf cfg.enable {
    services.udev.packages = [
      pkgs.qmk-udev-rules
    ]
    ++ lib.optionals cfg.keychronSupport [ pkgs.keychron-udev-rules ];
    users.groups.plugdev = { };
  };
}
