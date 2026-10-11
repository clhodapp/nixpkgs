{ lib, ... }:
{
  # The condition is false and its value fails when it is evaluated.
  # The check reports the definition all the same, and never evaluates
  # the value.
  greeting = lib.mkIf false (throw "the value of a definition must not be evaluated");
}
