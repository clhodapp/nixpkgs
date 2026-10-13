{
  config,
  lib,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/loader/grub/grub.nix
    # END generated imports
  ];

  options.hardware.facter.detected.uefi.supported = lib.mkEnableOption "the facter uefi module" // {
    default = config.hardware.facter.report.uefi.supported or false;
    defaultText = "hardware dependent";
  };

  config.boot.loader.grub.efiSupport = lib.mkIf config.hardware.facter.detected.uefi.supported (
    lib.mkDefault true
  );
}
