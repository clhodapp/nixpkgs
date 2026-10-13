{
  config,
  lib,
  pkgs,
  ...
}:

with lib;

let
  cfg = config.boot.loader.external;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../../../lib/modules/generic/meta-maintainers.nix
    ../../../../misc/meta.nix
    ../../../activation/activation-script.nix
    ../../stage-1.nix
    ../grub/grub.nix
    ../systemd-boot/systemd-boot.nix
    # END generated imports
  ];

  meta = {
    maintainers = with maintainers; [
      cole-h
      raitobezarius
    ];
    doc = ./external.md;
  };

  options.boot.loader.external = {
    enable = mkEnableOption "using an external tool to install your bootloader";

    installHook = mkOption {
      type = with types; path;
      description = ''
        The full path to a program of your choosing which performs the bootloader installation process.

        The program will be called with an argument pointing to the output of the system's toplevel.
      '';
    };
  };

  config = mkIf cfg.enable {
    boot.loader = {
      grub.enable = mkDefault false;
      systemd-boot.enable = mkDefault false;
      supportsInitrdSecrets = mkDefault false;
    };

    system.build.installBootLoader = cfg.installHook;
  };
}
