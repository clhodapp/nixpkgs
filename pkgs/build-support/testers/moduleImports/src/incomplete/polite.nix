{ lib, ... }:
lib.mkIf true {
  greeting = "good day";
}
