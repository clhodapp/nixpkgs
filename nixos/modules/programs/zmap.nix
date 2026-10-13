{
  pkgs,
  config,
  lib,
  ...
}:

let
  cfg = config.programs.zmap;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../system/etc/etc.nix
    # END generated imports
  ];

  options.programs.zmap = {
    enable = lib.mkEnableOption "ZMap, a network scanner designed for Internet-wide network surveys";
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ pkgs.zmap ];

    environment.etc."zmap/blacklist.conf".source = "${pkgs.zmap}/etc/zmap/blacklist.conf";
    environment.etc."zmap/zmap.conf".source = "${pkgs.zmap}/etc/zmap.conf";
  };
}
