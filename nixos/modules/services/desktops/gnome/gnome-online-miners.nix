# GNOME Online Miners daemon.

{ lib, ... }:

{
  imports = [
    (lib.mkRemovedOptionModule [
      "services"
      "gnome"
      "gnome-online-miners"
    ] "It was broken for a while now.")
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../../lib/modules/generic/assertions.nix
    # END generated imports
  ];
}
