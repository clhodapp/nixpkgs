{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.programs.ladybird;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/fonts/fontdir.nix
    ../config/system-path.nix
    # END generated imports
  ];

  options = {
    programs.ladybird.enable = lib.mkEnableOption "the Ladybird web browser";
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.ladybird ];
    fonts.fontDir.enable = true;
  };
}
