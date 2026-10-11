{ config, lib, ... }:
{
  options.message = lib.mkOption {
    type = lib.types.str;
    description = "The greeting with a name.";
  };

  config.message = "${config.greeting}, world";
}
