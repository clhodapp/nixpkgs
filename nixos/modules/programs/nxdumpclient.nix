{
  lib,
  pkgs,
  config,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.programs.nxdumpclient = {
    enable = lib.mkEnableOption "NX Dump Client";
  };

  config = lib.mkIf config.programs.nxdumpclient.enable {
    environment.systemPackages = [ pkgs.nxdumpclient ];
    services.udev.packages = [ pkgs.nxdumpclient ];
  };
}
