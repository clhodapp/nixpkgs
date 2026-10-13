{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.bpftune;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  meta = {
    maintainers = with lib.maintainers; [ nickcao ];
  };

  options = {
    services.bpftune = {
      enable = lib.mkEnableOption "bpftune BPF driven auto-tuning";

      package = lib.mkPackageOption pkgs "bpftune" { };
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.packages = [ cfg.package ];
    systemd.services.bpftune = {
      wantedBy = [ "multi-user.target" ];
      path = [ pkgs.kmod ]; # bpftune calls modprobe
    };
  };
}
