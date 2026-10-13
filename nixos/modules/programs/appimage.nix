{
  lib,
  config,
  pkgs,
  ...
}:

let
  cfg = config.programs.appimage;
in

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../system/boot/binfmt.nix
    ./fuse.nix
    # END generated imports
  ];

  options.programs.appimage = {
    enable = lib.mkEnableOption "appimage-run wrapper script for executing appimages on NixOS";
    binfmt = lib.mkEnableOption "binfmt registration to run appimages via appimage-run seamlessly";
    package = lib.mkPackageOption pkgs "appimage-run" {
      example = ''
        pkgs.appimage-run.override {
          extraPkgs = pkgs: [ pkgs.ffmpeg pkgs.imagemagick ];
        }
      '';
    };
  };

  config = lib.mkIf cfg.enable {
    boot.binfmt.registrations = lib.mkIf cfg.binfmt (
      let
        appimage_common = {
          wrapInterpreterInShell = false;
          interpreter = lib.getExe cfg.package;
          recognitionType = "magic";
          offset = 0;
          mask = ''\xff\xff\xff\xff\x00\x00\x00\x00\xff\xff\xff'';
        };
      in
      {
        appimage_type_1 = appimage_common // {
          magicOrExtension = ''\x7fELF....AI\x01'';
        };
        appimage_type_2 = appimage_common // {
          magicOrExtension = ''\x7fELF....AI\x02'';
        };
      }
    );
    environment.systemPackages = [ cfg.package ];

    programs.fuse.enable = true;
  };

  meta.maintainers = with lib.maintainers; [
    jopejoe1
    atemu
    aleksana
  ];
}
