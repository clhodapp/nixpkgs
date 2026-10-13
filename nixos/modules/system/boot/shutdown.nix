{
  config,
  ...
}:

{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../config/sysctl.nix
    ./systemd.nix
    # END generated imports
  ];

  boot.kernel.sysctl."kernel.poweroff_cmd" = "${config.systemd.package}/sbin/poweroff";

}
