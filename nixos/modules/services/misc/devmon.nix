{
  pkgs,
  config,
  lib,
  ...
}:
let
  cfg = config.services.devmon;

in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../hardware/udisks2.nix
    # END generated imports
  ];

  options = {
    services.devmon = {
      enable = lib.mkEnableOption "devmon, an automatic device mounting daemon";
    };
  };

  config = lib.mkIf cfg.enable {
    systemd.user.services.devmon = {
      description = "devmon automatic device mounting daemon";
      wantedBy = [ "default.target" ];
      path = [
        pkgs.udevil
        pkgs.procps
        pkgs.udisks
        pkgs.which
      ];
      serviceConfig.ExecStart = "${pkgs.udevil}/bin/devmon";
    };

    services.udisks2.enable = true;
  };
}
