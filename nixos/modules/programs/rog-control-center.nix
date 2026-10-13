{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.rog-control-center;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/hardware/asusd.nix
    # END generated imports
  ];

  options = {
    programs.rog-control-center = {
      enable = lib.mkEnableOption "the rog-control-center application";

      autoStart = lib.mkOption {
        default = false;
        type = lib.types.bool;
        description = "Whether rog-control-center should be started automatically.";
      };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [
      pkgs.asusctl
      (lib.mkIf cfg.autoStart (
        pkgs.makeAutostartItem {
          name = "rog-control-center";
          package = pkgs.asusctl;
        }
      ))
    ];

    services.asusd.enable = true;
  };

  meta.maintainers = pkgs.asusctl.meta.maintainers;
}
