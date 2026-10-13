{
  config,
  pkgs,
  lib,
  ...
}:
let
  inherit (lib) mkEnableOption mkIf;
  inherit (config.boot.kernelPackages) tt-kmd;

  cfg = config.hardware.tenstorrent;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.hardware.tenstorrent.enable = mkEnableOption "Tenstorrent driver & utilities";

  config = mkIf cfg.enable {
    boot = {
      extraModulePackages = [ tt-kmd ];
      kernelModules = [ "tenstorrent" ];
    };

    services.udev.packages = [
      tt-kmd
    ];

    environment.systemPackages = with pkgs; [
      tt-smi
      tt-system-tools
    ];
  };

  meta.maintainers = with lib.maintainers; [ RossComputerGuy ];
}
