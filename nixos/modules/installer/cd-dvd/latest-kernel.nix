{ lib, pkgs, ... }:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/boot/kernel.nix
    ../../system/etc/etc.nix
    ../../tasks/filesystems.nix
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
