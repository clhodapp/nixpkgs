{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.programs.xscreensaver.sonar;
  globalCfg = config.programs.xscreensaver;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../security/wrappers/default.nix
    ./xscreensaver.nix
    # END generated imports
  ];

  options.programs.xscreensaver.sonar = {
    enable = lib.mkEnableOption "xscreensaver";
  };
  config = lib.mkIf cfg.enable {
    security.wrappers.sonar = {
      capabilities = "cap_net_raw+ep";
      owner = "root";
      group = "root";
      source = pkgs.writeShellScript "sonar-fakeroot" ''
        exec ${lib.getExe pkgs.fakeroot} ${globalCfg.package}/libexec/xscreensaver/sonar "$@"
      '';
    };
  };
}
