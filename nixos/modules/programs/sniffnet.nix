{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.sniffnet;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options = {
    programs.sniffnet = {
      enable = lib.mkEnableOption "sniffnet, a network traffic monitor application";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.sniffnet ];

    security.wrappers.sniffnet = {
      owner = "root";
      group = "root";
      capabilities = "cap_net_raw,cap_net_admin=eip";
      source = "${pkgs.sniffnet}/bin/sniffnet";
    };
  };

  meta.maintainers = [ ];
}
