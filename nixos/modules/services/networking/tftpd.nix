{
  config,
  lib,
  pkgs,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./xinetd.nix
    # END generated imports
  ];

  options = {
    services.tftpd.enable = lib.mkOption {
      type = lib.types.bool;
      default = false;
      description = ''
        Whether to enable tftpd, a Trivial File Transfer Protocol server.
        The server will be run as an xinetd service.
      '';
    };

    services.tftpd.path = lib.mkOption {
      type = lib.types.path;
      default = "/srv/tftp";
      description = ''
        Where the tftp server files are stored.
      '';
    };
  };

  config = lib.mkIf config.services.tftpd.enable {
    services.xinetd.enable = true;

    services.xinetd.services = lib.singleton {
      name = "tftp";
      protocol = "udp";
      server = "${pkgs.netkittftp}/sbin/in.tftpd";
      serverArgs = "${config.services.tftpd.path}";
    };
  };
}
