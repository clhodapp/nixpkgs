# GNOME Online Miners daemon.

{ lib, ... }:

{
  imports = [
    (lib.mkRemovedOptionModule [
      "services"
      "gnome"
      "gnome-online-miners"
    ] "It was broken for a while now.")
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../../../../lib/modules/generic/assertions.nix
    # END generated imports
  ];
}
