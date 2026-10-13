{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.envision;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../services/hardware/udev.nix
    ../services/networking/avahi-daemon.nix
    ../services/networking/firewall.nix
    # END generated imports
  ];

  options = {
    programs.envision = {
      enable = lib.mkEnableOption "envision";

      package = lib.mkPackageOption pkgs "envision" { };

      openFirewall = lib.mkEnableOption "the default ports in the firewall for the WiVRn server" // {
        default = true;
      };
    };
  };

  config = lib.mkIf cfg.enable {
    services.avahi = {
      enable = true;
      publish = {
        enable = true;
        userServices = true;
      };
    };

    services.udev = {
      enable = true;
      packages = with pkgs; [
        xr-hardware
      ];
    };

    environment.systemPackages = [ cfg.package ];

    networking.firewall = lib.mkIf cfg.openFirewall {
      allowedTCPPorts = [ 9757 ];
      allowedUDPPorts = [ 9757 ];
    };
  };

  meta.maintainers = pkgs.envision.meta.maintainers;
}
