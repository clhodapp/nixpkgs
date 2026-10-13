{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.dublin-traceroute;

in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../security/wrappers/default.nix
    # END generated imports
  ];

  meta.maintainers = pkgs.dublin-traceroute.meta.maintainers;

  options = {
    programs.dublin-traceroute = {
      enable = lib.mkEnableOption "dublin-traceroute (including setcap wrapper)";

      package = lib.mkPackageOption pkgs "dublin-traceroute" { };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    security.wrappers.dublin-traceroute = {
      owner = "root";
      group = "root";
      capabilities = "cap_net_raw+p";
      source = lib.getExe cfg.package;
    };
  };
}
