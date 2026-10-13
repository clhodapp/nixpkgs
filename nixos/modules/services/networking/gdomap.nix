{
  config,
  lib,
  pkgs,
  ...
}:
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  #
  # interface
  #
  options = {
    services.gdomap = {
      enable = lib.mkEnableOption "GNUstep Distributed Objects name server";
    };
  };

  #
  # implementation
  #
  config = lib.mkIf config.services.gdomap.enable {
    # NOTE: gdomap runs as root
    # TODO: extra user for gdomap?
    systemd.services.gdomap = {
      description = "gdomap server";
      wantedBy = [ "multi-user.target" ];
      after = [ "network.target" ];
      path = [ pkgs.gnustep-base ];
      serviceConfig.ExecStart = "${pkgs.gnustep-base}/bin/gdomap -f";
    };
  };
}
