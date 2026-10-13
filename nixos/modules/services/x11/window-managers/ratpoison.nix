{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.services.xserver.windowManager.ratpoison;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  ###### interface
  options = {
    services.xserver.windowManager.ratpoison.enable = mkEnableOption "ratpoison";
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "ratpoison";
      start = ''
        ${pkgs.ratpoison}/bin/ratpoison &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ pkgs.ratpoison ];
  };
}
