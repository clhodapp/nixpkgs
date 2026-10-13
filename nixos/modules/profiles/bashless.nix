{ lib, ... }:

{

  # Bashless builds on perlless
  imports = [
    ./perlless.nix
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/console.nix
    ../config/nix.nix
    ../config/power-management.nix
    ../config/shells-environment.nix
    ../config/system-path.nix
    ../config/users-groups.nix
    ../misc/man-db.nix
    ../programs/bash/bash.nix
    ../security/wrappers/default.nix
    ../services/networking/firewall.nix
    ../system/activation/activatable-system.nix
    ../system/activation/nixos-init.nix
    ../system/activation/top-level.nix
    ../system/boot/kexec.nix
    ../tasks/bcache.nix
    ../tasks/lvm.nix
    ../tasks/network-interfaces.nix
    # END generated imports
  ];

  # Remove bash from activation
  system.nixos-init.enable = lib.mkDefault true;
  system.activatable = lib.mkDefault false;
  environment.shell.enable = lib.mkDefault false;
  programs.bash.enable = lib.mkDefault false;

  # Random bash remnants
  environment.corePackages = lib.mkForce [ ];
  # Contains bash completions
  nix.enable = lib.mkDefault false;
  documentation.man.man-db.enable = lib.mkDefault false;
  # autovt depends on bash
  console.enable = lib.mkDefault false;
  # dhcpcd and openresolv depend on bash
  networking.useNetworkd = lib.mkDefault true;
  # bcache tools depend on bash.
  boot.bcache.enable = lib.mkDefault false;
  # iptables depends on bash and nixos-firewall-tool is a bash script
  networking.firewall.enable = lib.mkDefault false;
  # the wrapper script is in bash
  security.enableWrappers = lib.mkDefault false;
  # kexec script is written in bash
  boot.kexec.enable = lib.mkDefault false;
  # Relies on bash scripts
  powerManagement.enable = lib.mkDefault false;
  users.manageLingering = lib.mkDefault false;
  # The resize helper pulls in bash
  services.lvm.resizeHelper.enable = false;

  # Check that the system does not contain a Nix store path that contains the
  # string "bash".
  system.forbiddenDependenciesRegexes = [ "bash" ];

}
