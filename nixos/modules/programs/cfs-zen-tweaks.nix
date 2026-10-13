# CFS Zen Tweaks

{
  config,
  pkgs,
  lib,
  ...
}:

let

  cfg = config.programs.cfs-zen-tweaks;

in

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../system/boot/systemd.nix
    # END generated imports
  ];

  meta = {
    maintainers = with lib.maintainers; [ mkg20001 ];
  };

  options = {
    programs.cfs-zen-tweaks.enable = lib.mkEnableOption "CFS Zen Tweaks";
  };

  config = lib.mkIf cfg.enable {
    systemd.packages = [ pkgs.cfs-zen-tweaks ];

    systemd.services.set-cfs-tweaks.wantedBy = [
      "multi-user.target"
      "suspend.target"
      "hibernate.target"
      "hybrid-sleep.target"
      "suspend-then-hibernate.target"
    ];
  };
}
