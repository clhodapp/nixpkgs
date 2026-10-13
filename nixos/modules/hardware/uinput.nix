{ config, lib, ... }:

let
  cfg = config.hardware.uinput;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/users-groups.nix
    ../services/hardware/udev.nix
    ../system/boot/kernel.nix
    # END generated imports
  ];

  options.hardware.uinput = {
    enable = lib.mkEnableOption "uinput support";
  };

  config = lib.mkIf cfg.enable {
    boot.kernelModules = [ "uinput" ];

    users.groups.uinput = { };

    services.udev.extraRules = ''
      SUBSYSTEM=="misc", KERNEL=="uinput", MODE="0660", GROUP="uinput", OPTIONS+="static_node=uinput"
    '';
  };
}
