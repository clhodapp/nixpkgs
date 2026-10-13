{
  lib,
  config,
  pkgs,
  ...
}:
let
  cfg = config.programs.bash.blesh;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ./bash.nix
    # END generated imports
  ];

  options = {
    programs.bash.blesh.enable = lib.mkEnableOption "blesh, a full-featured line editor written in pure Bash";
  };

  config = lib.mkIf cfg.enable {
    programs.bash.interactiveShellInit = lib.mkBefore ''
      source ${pkgs.blesh}/share/blesh/ble.sh
    '';
  };
  meta.maintainers = with lib.maintainers; [ laalsaas ];
}
