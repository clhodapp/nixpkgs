{ lib, ... }:

{
  imports = [
    (lib.mkRemovedOptionModule [ "security" "klogd" "enable" ] ''
      Logging of kernel messages is now handled by systemd.
    '')
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/assertions.nix
    # END generated imports
  ];
}
