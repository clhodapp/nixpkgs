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
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../config/fonts/fontdir.nix
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
