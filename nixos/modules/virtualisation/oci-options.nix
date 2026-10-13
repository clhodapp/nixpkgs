{
  lib,
  ...
}:
{
  imports = [
    ./disk-size-option.nix
    (lib.mkRenamedOptionModuleWith {
      sinceRelease = 2411;
      from = [
        "oci"
        "diskSize"
      ];
      to = [
        "virtualisation"
        "diskSize"
      ];
    })
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/assertions.nix
    # END generated imports
  ];

  options = {
    oci = {
      efi = lib.mkOption {
        default = true;
        internal = true;
        description = ''
          Whether the OCI instance is using EFI.
        '';
      };
    };
  };
}
