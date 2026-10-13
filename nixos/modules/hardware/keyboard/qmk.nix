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
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/users-groups.nix
    ../../services/hardware/udev.nix
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
