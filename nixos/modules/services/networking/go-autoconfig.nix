{
  config,
  lib,
  pkgs,
  ...
}:
let

  cfg = config.services.go-autoconfig;
  format = pkgs.formats.yaml { };
  configFile = format.generate "config.yml" cfg.settings;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../system/boot/systemd.nix
    # END generated imports
  ];

  options = {
    services.go-autoconfig = {

      enable = lib.mkEnableOption "IMAP/SMTP autodiscover feature for mail clients";

      settings = lib.mkOption {
        default = { };
        description = ''
          Configuration for go-autoconfig. See
          <https://github.com/L11R/go-autoconfig/blob/master/config.yml>
          for more information.
        '';
        type = lib.types.submodule {
          freeformType = format.type;
        };
        example = lib.literalExpression ''
          {
            service_addr = ":1323";
            domain = "autoconfig.example.org";
            imap = {
              server = "example.org";
              port = 993;
            };
            smtp = {
              server = "example.org";
              port = 465;
            };
          }
        '';
      };

    };
  };

  config = lib.mkIf cfg.enable {

    systemd = {
      services.go-autoconfig = {
        wantedBy = [ "multi-user.target" ];
        description = "IMAP/SMTP autodiscover server";
        after = [ "network.target" ];
        serviceConfig = {
          ExecStart = "${pkgs.go-autoconfig}/bin/go-autoconfig -config ${configFile}";
          Restart = "on-failure";
          WorkingDirectory = "${pkgs.go-autoconfig}/";
          DynamicUser = true;
        };
      };
    };

  };

  meta.maintainers = with lib.maintainers; [ onny ];

}
