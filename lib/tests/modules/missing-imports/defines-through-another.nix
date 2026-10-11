{ lib, ... }:
{
  # `defines-with-import.nix` imports the module that declares
  # `greeting`, so this module reaches the declaration through it.
  imports = [ ./defines-with-import.nix ];
  greeting = lib.mkDefault "good day";
}
