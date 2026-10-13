# Global configuration for mininet
# kernel must have NETNS/VETH/SCHED
{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.mininet;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../virtualisation/openvswitch.nix
    # END generated imports
  ];

  options.programs.mininet.enable = lib.mkEnableOption "Mininet, an emulator for rapid prototyping of Software Defined Networks";

  config = lib.mkIf cfg.enable {

    virtualisation.vswitch.enable = true;

    environment.systemPackages = [ pkgs.mininet ];
  };
}
