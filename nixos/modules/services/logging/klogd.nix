{ lib, ... }:

{
  imports = [
    (lib.mkRemovedOptionModule [ "security" "klogd" "enable" ] ''
      Logging of kernel messages is now handled by systemd.
    '')
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../../../lib/modules/generic/assertions.nix
    # END generated imports
  ];
}
