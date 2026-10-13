{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.kbdlight;

in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options.programs.kbdlight.enable = lib.mkEnableOption "kbdlight";

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.kbdlight ];
    security.wrappers.kbdlight = {
      setuid = true;
      owner = "root";
      group = "root";
      source = "${pkgs.kbdlight.out}/bin/kbdlight";
    };
  };
}
