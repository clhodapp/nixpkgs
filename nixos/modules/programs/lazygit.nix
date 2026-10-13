{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.lazygit;

  settingsFormat = pkgs.formats.yaml { };
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../system/etc/etc.nix
    # END generated imports
  ];

  options.programs.lazygit = {
    enable = lib.mkEnableOption "lazygit, a simple terminal UI for git commands";

    package = lib.mkPackageOption pkgs "lazygit" { };

    settings = lib.mkOption {
      inherit (settingsFormat) type;
      default = { };
      description = ''
        Lazygit configuration.

        See <https://github.com/jesseduffield/lazygit/blob/master/docs/Config.md> for documentation.
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    environment = {
      systemPackages = [ cfg.package ];
      etc = lib.mkIf (cfg.settings != { }) {
        "xdg/lazygit/config.yml".source = settingsFormat.generate "lazygit-config.yml" cfg.settings;
      };
    };
  };

  meta = {
    maintainers = with lib.maintainers; [ linsui ];
  };
}
