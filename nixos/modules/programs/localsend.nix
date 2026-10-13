{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.localsend;
  firewallPort = 53317;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/networking/firewall.nix
    # END generated imports
  ];

  options.programs.localsend = {
    enable = lib.mkEnableOption "localsend, an open source cross-platform alternative to AirDrop";

    package = lib.mkPackageOption pkgs "localsend" { };

    openFirewall =
      lib.mkEnableOption "opening the firewall port ${toString firewallPort} for receiving files"
      // {
        default = true;
      };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    networking.firewall.allowedTCPPorts = lib.optionals cfg.openFirewall [ firewallPort ];
    networking.firewall.allowedUDPPorts = lib.optionals cfg.openFirewall [ firewallPort ];
  };

  meta.maintainers = with lib.maintainers; [ pandapip1 ];
}
