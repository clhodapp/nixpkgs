{ lib, ... }:

{
  imports = [
    ./installation-cd-minimal.nix
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../system/activation/specialisation.nix
    # END generated imports
  ];

  isoImage.configurationName = lib.mkDefault "(Linux LTS)";

  specialisation.latest_kernel.configuration =
    { config, ... }:
    {
      imports = [ ./latest-kernel.nix ];
      isoImage.configurationName = "(Linux ${config.boot.kernelPackages.kernel.version})";
    };
}
