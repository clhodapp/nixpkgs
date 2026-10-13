{ lib, ... }:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../pcmcia.nix
    # END generated imports
  ];

  hardware = {
    pcmcia = {
      firmware = [ (lib.cleanSource ./firmware) ];
    };
  };
}
