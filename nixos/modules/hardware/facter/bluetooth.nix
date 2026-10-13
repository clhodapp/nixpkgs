{ lib, config, ... }:
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../services/hardware/bluetooth.nix
    # END generated imports
  ];

  options.hardware.facter.detected.bluetooth.enable =
    lib.mkEnableOption "Enable the Facter bluetooth module"
    // {
      default = builtins.length (config.hardware.facter.report.hardware.bluetooth or [ ]) > 0;
      defaultText = "hardware dependent";
    };

  config.hardware.bluetooth.enable = lib.mkIf config.hardware.facter.detected.bluetooth.enable (
    lib.mkDefault true
  );
}
