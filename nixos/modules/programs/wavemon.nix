{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.wavemon;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options = {
    programs.wavemon = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to add wavemon to the global environment and configure a
          setcap wrapper for it.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    environment.systemPackages = with pkgs; [ wavemon ];
    security.wrappers.wavemon = {
      owner = "root";
      group = "root";
      capabilities = "cap_net_admin+ep";
      source = "${pkgs.wavemon}/bin/wavemon";
    };
  };
}
