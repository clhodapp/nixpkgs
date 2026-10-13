{
  config,
  pkgs,
  lib,
  ...
}:

let
  cfg = config.programs.git-worktree-switcher;

  initScript =
    shell:
    if (shell == "fish") then
      ''
        ${lib.getExe cfg.package} init ${shell} | source
      ''
    else
      ''
        eval "$(${lib.getExe cfg.package} init ${shell})"
      '';
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./bash/bash.nix
    # END generated imports
  ];

  options = {
    programs.git-worktree-switcher = {
      enable = lib.mkEnableOption "git-worktree-switcher, switch between git worktrees with speed.";
      package = lib.mkPackageOption pkgs "git-worktree-switcher" { };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [ git-worktree-switcher ];

    programs.bash.interactiveShellInit = initScript "bash";
    programs.zsh.interactiveShellInit = lib.optionalString config.programs.zsh.enable (
      initScript "zsh"
    );
    programs.fish.interactiveShellInit = lib.optionalString config.programs.fish.enable (
      initScript "fish"
    );
  };
}
