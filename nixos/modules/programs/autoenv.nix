{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.autoenv;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ./bash/bash.nix
    ./zsh/zsh.nix
    # END generated imports
  ];

  options = {
    programs.autoenv = {
      enable = lib.mkEnableOption "autoenv";
      package = lib.mkPackageOption pkgs "autoenv" { };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.autoenv ];

    programs = {
      zsh.interactiveShellInit = ''
        source ${cfg.package}/share/autoenv/activate.sh
      '';

      bash.interactiveShellInit = ''
        source ${cfg.package}/share/autoenv/activate.sh
      '';
    };
  };
}
