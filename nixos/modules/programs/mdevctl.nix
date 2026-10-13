{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.programs.mdevctl;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../system/etc/etc.nix
    # END generated imports
  ];

  options.programs.mdevctl = {
    enable = lib.mkEnableOption "Mediated Device Management";
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [ mdevctl ];

    environment.etc."mdevctl.d/.keep".text = "";
    environment.etc."mdevctl/scripts.d/notifiers/.keep".text = "";
    environment.etc."mdevctl/scripts.d/callouts/.keep".text = "";

  };
}
