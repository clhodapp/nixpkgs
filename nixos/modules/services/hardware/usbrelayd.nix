{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.services.usbrelayd;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/meta-maintainers.nix
    ../../config/users-groups.nix
    ../../system/boot/systemd.nix
    ../../system/etc/etc.nix
    ./udev.nix
    # END generated imports
  ];

  options.services.usbrelayd = with lib.types; {
    enable = lib.mkEnableOption "USB Relay MQTT daemon";

    broker = lib.mkOption {
      type = str;
      description = "Hostname or IP address of your MQTT Broker.";
      default = "127.0.0.1";
      example = [
        "mqtt"
        "192.168.1.1"
      ];
    };

    clientName = lib.mkOption {
      type = str;
      description = "Name, your client connects as.";
      default = "MyUSBRelay";
    };
  };

  config = lib.mkIf cfg.enable {

    environment.etc."usbrelayd.conf".text = ''
      [MQTT]
      BROKER = ${cfg.broker}
      CLIENTNAME = ${cfg.clientName}
    '';

    services.udev.packages = [ pkgs.usbrelayd ];
    systemd.packages = [ pkgs.usbrelayd ];
    users.groups.usbrelay = { };
  };

  meta = {
    maintainers = with lib.maintainers; [ wentasah ];
  };
}
