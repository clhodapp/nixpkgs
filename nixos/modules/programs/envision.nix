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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/hardware/udev.nix
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
