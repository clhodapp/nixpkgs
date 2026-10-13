{
  config,
  lib,
  pkgs,
  ...
}:

let
  inherit (lib)
    mkEnableOption
    mkIf
    mkPackageOption
    singleton
    ;
  cfg = config.services.xserver.windowManager.katriawm;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./default.nix
    # END generated imports
  ];

  ###### interface
  options = {
    services.xserver.windowManager.katriawm = {
      enable = mkEnableOption "katriawm";
      package = mkPackageOption pkgs "katriawm" { };
    };
  };

  ###### implementation
  config = mkIf cfg.enable {
    services.xserver.windowManager.session = singleton {
      name = "katriawm";
      start = ''
        ${cfg.package}/bin/katriawm &
        waitPID=$!
      '';
    };
    environment.systemPackages = [ cfg.package ];
  };
}
