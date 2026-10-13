{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.liboping;
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options.programs.liboping = {
    enable = lib.mkEnableOption "liboping";
  };
  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [ liboping ];
    security.wrappers = lib.mkMerge (
      map
        (exec: {
          "${exec}" = {
            owner = "root";
            group = "root";
            capabilities = "cap_net_raw+p";
            source = "${pkgs.liboping}/bin/${exec}";
          };
        })
        [
          "oping"
          "noping"
        ]
    );
  };
}
