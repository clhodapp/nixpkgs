{ config, lib, ... }:
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../systemd.nix
    # END generated imports
  ];

  options.boot.loader.efi = {

    canTouchEfiVariables = lib.mkOption {
      default = false;
      type = lib.types.bool;
      description = "Whether the installation process is allowed to modify EFI boot variables.";
    };

    efiSysMountPoint = lib.mkOption {
      default = "/boot";
      type = lib.types.str;
      description = "Where the EFI System Partition is mounted.";
    };
  };

  config = {
    # Applied to all bootloader support random seed initialization
    systemd.services.systemd-boot-random-seed.environment.SYSTEMD_ESP_PATH =
      config.boot.loader.efi.efiSysMountPoint;
  };
}
