{
  config,
  pkgs,
  lib,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../config/system-path.nix
    ../services/system/dbus.nix
    ../system/boot/kernel.nix
    ../system/boot/systemd.nix
    # END generated imports
  ];

  options = {
    programs.appgate-sdp = {
      enable = lib.mkEnableOption "the AppGate SDP VPN client";
    };
  };

  config = lib.mkIf config.programs.appgate-sdp.enable {
    boot.kernelModules = [ "tun" ];
    environment.systemPackages = [ pkgs.appgate-sdp ];
    services.dbus.packages = [ pkgs.appgate-sdp ];
    systemd = {
      packages = [ pkgs.appgate-sdp ];
      # https://github.com/NixOS/nixpkgs/issues/81138
      services.appgatedriver.wantedBy = [ "multi-user.target" ];
      services.appgate-dumb-resolver.path = [ pkgs.e2fsprogs ];
      services.appgate-resolver.path = [
        pkgs.procps
        pkgs.e2fsprogs
      ];
      services.appgatedriver.path = [ pkgs.e2fsprogs ];
    };
  };
}
