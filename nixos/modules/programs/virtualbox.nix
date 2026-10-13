let
  msg =
    "Importing <nixpkgs/nixos/modules/programs/virtualbox.nix> is "
    + "deprecated, please use `virtualisation.virtualbox.host.enable = true' "
    + "instead.";
in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../virtualisation/virtualbox-host.nix
    # END generated imports
  ];

  config.warnings = [ msg ];
  config.virtualisation.virtualbox.host.enable = true;
}
