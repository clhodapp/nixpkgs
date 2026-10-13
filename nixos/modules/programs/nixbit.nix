{
  config,
  lib,
  pkgs,
  ...
}:

let
  inherit (lib)
    mkEnableOption
    mkIf
    mkOption
    mkPackageOption
    types
    ;
  cfg = config.programs.nixbit;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../system/etc/etc.nix
    # END generated imports
  ];

  options.programs.nixbit = {
    enable = mkEnableOption "Nixbit configuration";

    package = mkPackageOption pkgs "nixbit" { };

    repository = mkOption {
      type = types.str;
      description = "Git repository URL for Nixbit";
    };

    forceAutostart = mkEnableOption "" // {
      description = "Force creation of autostart desktop entry when application starts";
    };
  };

  config = mkIf cfg.enable {
    environment = {
      systemPackages = [ cfg.package ];

      etc."nixbit.conf".text =
        lib.optionalString (cfg.repository != "") ''
          [Repository]
          Url = ${cfg.repository}
        ''
        + ''
          [Autostart]
          Force = ${if cfg.forceAutostart then "true" else "false"}
        '';
    };
  };
}
