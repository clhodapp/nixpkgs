{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.xastir;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options.programs.xastir = {
    enable = lib.mkEnableOption "Xastir Graphical APRS client";
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [ xastir ];
    security.wrappers.xastir = {
      source = "${pkgs.xastir}/bin/xastir";
      capabilities = "cap_net_raw+p";
      owner = "root";
      group = "root";
    };
  };
}
