{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.tcpdump;
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../config/users-groups.nix
    # END generated imports
  ];

  options = {
    programs.tcpdump = {
      enable = lib.mkOption {
        type = lib.types.bool;
        default = false;
        description = ''
          Whether to configure a setcap wrapper for tcpdump.
          To use it, add your user to the `pcap` group.
        '';
      };
    };
  };

  config = lib.mkIf cfg.enable {
    security.wrappers.tcpdump = {
      owner = "root";
      group = "pcap";
      capabilities = "cap_net_raw+p";
      permissions = "u+rx,g+x";
      source = lib.getExe pkgs.tcpdump;
    };

    users.groups.pcap = { };
  };
}
