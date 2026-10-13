{
  config,
  pkgs,
  lib,
  ...
}:

let
  prg = config.programs;
  cfg = prg.bash-my-aws;

  initScript = ''
    eval $(${pkgs.bash-my-aws}/bin/bma-init)
  '';
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./bash/bash.nix
    # END generated imports
  ];

  options = {
    programs.bash-my-aws = {
      enable = lib.mkEnableOption "bash-my-aws";
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [ bash-my-aws ];

    programs.bash.interactiveShellInit = initScript;
  };
}
