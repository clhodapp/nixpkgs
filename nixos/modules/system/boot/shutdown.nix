{
  config,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../config/sysctl.nix
    # END generated imports
  ];

  boot.kernel.sysctl."kernel.poweroff_cmd" = "${config.systemd.package}/sbin/poweroff";

}
