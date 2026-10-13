{ lib, pkgs, ... }:
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../system/boot/kernel.nix
    # END generated imports
  ];

  boot.kernelPackages = pkgs.linuxPackages_latest;
  boot.supportedFilesystems.zfs = false;
  boot.supportedFilesystems.bcachefs = true;
  environment.etc."nixos-generate-config.conf".text = ''
    [Defaults]
    Kernel=latest
  '';
}
