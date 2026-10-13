{
  pkgs,
  config,
  lib,
  ...
}:
let
  inherit (lib)
    mkEnableOption
    mkPackageOption
    mkRemovedOptionModule
    optionalString
    mkIf
    ;
  cfg = config.programs.skim;
in
{
  imports = [
    (mkRemovedOptionModule [ "programs" "skim" "fuzzyCompletion" ]
      "programs.skim.fuzzyCompletion has been removed. Completions are now included in the package itself."
    )
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/assertions.nix
    ../config/system-path.nix
    ./bash/bash.nix
    ./fish.nix
    ./zsh/zsh.nix
    # END generated imports
  ];

  options = {
    programs.skim = {
      enable = mkEnableOption "skim fuzzy finder";
      keybindings = mkEnableOption "skim keybindings";
      package = mkPackageOption pkgs "skim" { };
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    programs.bash.interactiveShellInit = optionalString cfg.keybindings ''
      source ${cfg.package}/share/skim/key-bindings.bash
    '';

    programs.zsh.interactiveShellInit = optionalString cfg.keybindings ''
      source ${cfg.package}/share/skim/key-bindings.zsh
    '';

    programs.fish.interactiveShellInit = optionalString cfg.keybindings ''
      source ${cfg.package}/share/skim/key-bindings.fish && skim_key_bindings
    '';
  };
}
