{ lib, ... }:

{
  imports = [
    (lib.mkRemovedOptionModule [
      "programs"
      "qt5ct"
      "enable"
    ] "Use qt5.platformTheme = \"qt5ct\" instead.")
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/assertions.nix
    # END generated imports
  ];
}
