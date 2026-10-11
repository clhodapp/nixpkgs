{ config, lib, ... }:
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ./greeting.nix
    # END generated imports
  ];

  options.message = lib.mkOption {
    type = lib.types.str;
    description = "The greeting with a name.";
  };

  config.message = "${config.greeting}, world";
}
