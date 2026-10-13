{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let

  configFile = ./xfs.conf;

in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/assertions.nix
    ../../config/fonts/fontdir.nix
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  ###### interface

  options = {

    services.xfs = {

      enable = mkOption {
        type = types.bool;
        default = false;
        description = "Whether to enable the X Font Server.";
      };

    };

  };

  ###### implementation

  config = mkIf config.services.xfs.enable {
    assertions = singleton {
      assertion = config.fonts.enableFontDir;
      message = "Please enable fonts.enableFontDir to use the X Font Server.";
    };

    systemd.services.xfs = {
      description = "X Font Server";
      after = [ "network.target" ];
      wantedBy = [ "multi-user.target" ];
      path = [ pkgs.xfs ];
      script = "xfs -config ${configFile}";
    };
  };
}
