{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.pulseview;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/hardware/udev.nix
    # END generated imports
  ];

  options.programs.pulseview = {
    enable = lib.mkEnableOption "pulseview, a sigrok GUI";
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [
      pkgs.pulseview
    ];

    services.udev = {
      packages = [
        # Pulseview needs some udev rules provided by libsigrok to access devices
        pkgs.libsigrok
      ];
    };
  };
}
