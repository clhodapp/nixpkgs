{ lib, ... }:

{
  imports = [
    ./installation-cd-minimal-new-kernel.nix
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../tasks/filesystems.nix
    # END generated imports
  ];

  boot.supportedFilesystems.zfs = lib.mkForce false;
}
