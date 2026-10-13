# Common configuration for headless machines (e.g., Amazon EC2
# instances).

{ lib, ... }:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../system/boot/emergency-mode.nix
    ../system/boot/kernel.nix
    ../system/boot/loader/grub/grub.nix
    ../system/boot/systemd.nix
    # END generated imports
  ];

  # Don't start a tty on the serial consoles.
  systemd.services."serial-getty@ttyS0".enable = lib.mkDefault false;
  systemd.services."serial-getty@hvc0".enable = false;
  systemd.services."getty@tty1".enable = false;
  systemd.services."autovt@".enable = false;

  # Since we can't manually respond to a panic, just reboot.
  boot.kernelParams = [
    "panic=1"
    "boot.panic_on_fail"
    "vga=0x317"
    "nomodeset"
  ];

  # Don't allow emergency mode, because we don't have a console.
  systemd.enableEmergencyMode = false;

  # Being headless, we don't need a GRUB splash image.
  boot.loader.grub.splashImage = null;
}
