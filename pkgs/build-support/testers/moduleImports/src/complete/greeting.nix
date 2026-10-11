{ lib, ... }:
{
  options.greeting = lib.mkOption {
    type = lib.types.str;
    default = "hello";
    description = "The greeting.";
  };
}
