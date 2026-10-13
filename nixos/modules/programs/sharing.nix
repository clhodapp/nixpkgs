{
  config,
  pkgs,
  lib,
  ...
}:
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/networking/firewall.nix
    # END generated imports
  ];

  options.programs.sharing = {
    enable = lib.mkEnableOption ''
      sharing, a CLI tool for sharing files.

      Note that it will opens the 7478 port for TCP in the firewall, which is needed for it to function properly
    '';
  };
  config =
    let
      cfg = config.programs.sharing;
    in
    lib.mkIf cfg.enable {
      environment.systemPackages = [ pkgs.sharing ];
      networking.firewall.allowedTCPPorts = [ 7478 ];
    };
}
