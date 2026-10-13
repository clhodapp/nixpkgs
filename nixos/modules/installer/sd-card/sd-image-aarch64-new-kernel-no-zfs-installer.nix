{ lib, ... }:

{
  imports = [
    ./sd-image-aarch64-new-kernel-installer.nix
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../tasks/filesystems.nix
    # END generated imports
  ];

  boot.supportedFilesystems.zfs = lib.mkForce false;
}
