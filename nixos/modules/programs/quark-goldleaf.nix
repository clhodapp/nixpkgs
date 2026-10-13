{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.quark-goldleaf;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options = {
    programs.quark-goldleaf = {
      enable = lib.mkEnableOption "quark-goldleaf with udev rules applied";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.quark-goldleaf ];
    services.udev.packages = [ pkgs.quark-goldleaf ];
  };

  meta.maintainers = pkgs.quark-goldleaf.meta.maintainers;
}
