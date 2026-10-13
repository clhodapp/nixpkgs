{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.npm;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/shells-environment.nix
    ../config/system-path.nix
    ../system/etc/etc.nix
    # END generated imports
  ];

  ###### interface

  options = {
    programs.npm = {
      enable = lib.mkEnableOption "{command}`npm` global config";

      package = lib.mkPackageOption pkgs "nodejs" { };

      npmrc = lib.mkOption {
        type = lib.types.lines;
        description = ''
          The system-wide npm configuration.
          See <https://docs.npmjs.com/misc/config>.
        '';
        default = ''
          prefix = ''${HOME}/.npm
        '';
        example = ''
          prefix = ''${HOME}/.npm
          https-proxy=proxy.example.com
          init-license=MIT
          init-author-url=https://www.npmjs.com/
          color=true
        '';
      };
    };
  };

  ###### implementation

  config = lib.mkIf cfg.enable {
    environment.etc.npmrc.text = cfg.npmrc;

    environment.variables.NPM_CONFIG_GLOBALCONFIG = "/etc/npmrc";

    environment.systemPackages = [ cfg.package ];
  };

}
