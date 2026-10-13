# This module defines a NixOS configuration with the Plasma 6 desktop.
# It's used by the graphical installation CD.

{ pkgs, ... }:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/desktop-managers/plasma6.nix
    # END generated imports
  ];

  services.xserver = {
    enable = true;
    desktopManager.plasma6.enable = true;
  };

  services = {
    displayManager.plasma-login-manager.enable = true;
    libinput.enable = true; # for touchpad support on many laptops
  };

  environment.systemPackages = [
    pkgs.mesa-demos
    pkgs.firefox
  ];
}
