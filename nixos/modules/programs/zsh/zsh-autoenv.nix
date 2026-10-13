{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.zsh.zsh-autoenv;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./zsh.nix
    # END generated imports
  ];

  options = {
    programs.zsh.zsh-autoenv = {
      enable = lib.mkEnableOption "zsh-autoenv";
      package = lib.mkPackageOption pkgs "zsh-autoenv" { };
    };
  };

  config = lib.mkIf cfg.enable {
    programs.zsh.interactiveShellInit = ''
      source ${cfg.package}/share/zsh-autoenv/autoenv.zsh
    '';
  };
}
