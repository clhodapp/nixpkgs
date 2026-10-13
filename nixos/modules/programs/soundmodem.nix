{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.soundmodem;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../config/users-groups.nix
    # END generated imports
  ];

  options = {
    programs.soundmodem = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to add Soundmodem to the global environment and configure a
          wrapper for 'soundmodemconfig' for users in the 'soundmodem' group.
        '';
      };
      package = lib.mkPackageOption pkgs "soundmodem" { };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];
    users.groups.soundmodem = { };

    security.wrappers.soundmodemconfig = {
      source = "${cfg.package}/bin/soundmodemconfig";
      owner = "root";
      group = "soundmodem";
      permissions = "u+rx,g+x";
    };
  };
}
