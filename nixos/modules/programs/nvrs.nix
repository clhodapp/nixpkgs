{
  lib,
  pkgs,
  config,
  ...
}:

let
  cfg = config.programs.nvrs;
  settingsFormat = pkgs.formats.toml { };
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../system/etc/etc.nix
    # END generated imports
  ];

  meta.maintainers = with lib.maintainers; [ koi ];

  options.programs.nvrs = {
    enable = lib.mkEnableOption "nvrs";

    package = lib.mkPackageOption pkgs "nvrs" { };

    settings = lib.mkOption {
      type = lib.types.submodule {
        freeformType = settingsFormat.type;
      };

      default = { };
      description = ''
        Configuration written to {file}`$XDG_CONFIG_HOME/nvrs/config.toml`

        See <https://nvrs.adamperkowski.dev/configuration.html> for details.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    environment.etc = {
      "nvrs/nvrs.toml" = lib.mkIf (cfg.settings != { }) {
        source = settingsFormat.generate "nvrs-config.toml" cfg.settings;
      };
    };
  };
}
