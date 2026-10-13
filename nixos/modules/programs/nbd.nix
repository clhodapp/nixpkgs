{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.nbd;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../system/boot/kernel.nix
    # END generated imports
  ];

  options = {
    programs.nbd = {
      enable = lib.mkEnableOption "Network Block Device (nbd) support";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [ nbd ];
    boot.kernelModules = [ "nbd" ];
  };
}
