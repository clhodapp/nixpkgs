{
  config,
  lib,
  pkgs,
  ...
}:

with lib;
let
  dmcfg = config.services.xserver.displayManager;
  ldmcfg = dmcfg.lightdm;
  cfg = ldmcfg.greeters.mobile;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./gtk.nix
    # END generated imports
  ];

  options = {
    services.xserver.displayManager.lightdm.greeters.mobile = {
      enable = mkEnableOption "lightdm-mobile-greeter as the lightdm greeter";
    };
  };

  config = mkIf (ldmcfg.enable && cfg.enable) {
    services.xserver.displayManager.lightdm.greeters.gtk.enable = false;

    services.xserver.displayManager.lightdm.greeter = mkDefault {
      package = pkgs.lightdm-mobile-greeter.xgreeters;
      name = "lightdm-mobile-greeter";
    };
  };
}
