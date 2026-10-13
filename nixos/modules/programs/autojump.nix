{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.autojump;
  prg = config.programs;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ./bash/bash.nix
    ./fish.nix
    ./zsh/zsh.nix
    # END generated imports
  ];

  options = {
    programs.autojump = {

      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to enable autojump.
        '';
      };
    };
  };

  ###### implementation

  config = lib.mkIf cfg.enable {
    environment.pathsToLink = [ "/share/autojump" ];
    environment.systemPackages = [ pkgs.autojump ];

    programs.bash.interactiveShellInit = "source ${pkgs.autojump}/share/autojump/autojump.bash";
    programs.zsh.interactiveShellInit = lib.mkIf prg.zsh.enable "source ${pkgs.autojump}/share/autojump/autojump.zsh";
    programs.fish.interactiveShellInit = lib.mkIf prg.fish.enable "source ${pkgs.autojump}/share/autojump/autojump.fish";
  };
}
