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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
