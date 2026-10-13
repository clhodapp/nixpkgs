{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.ausweisapp;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/networking/firewall.nix
    # END generated imports
  ];

  options.programs.ausweisapp = {
    enable = lib.mkEnableOption "AusweisApp";

    openFirewall = lib.mkOption {
      description = ''
        Whether to open the required firewall ports for the Smartphone as Card Reader (SaC) functionality of AusweisApp.
      '';
      default = false;
      type = lib.types.bool;
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [ ausweisapp ];
    networking.firewall.allowedUDPPorts = lib.optionals cfg.openFirewall [ 24727 ];
  };
}
