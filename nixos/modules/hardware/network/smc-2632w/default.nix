{ lib, ... }:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../pcmcia.nix
    # END generated imports
  ];

  hardware = {
    pcmcia = {
      firmware = [ (lib.cleanSource ./firmware) ];
    };
  };
}
