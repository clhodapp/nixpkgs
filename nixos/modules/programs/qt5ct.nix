{ lib, ... }:

{
  imports = [
    (lib.mkRemovedOptionModule [
      "programs"
      "qt5ct"
      "enable"
    ] "Use qt5.platformTheme = \"qt5ct\" instead.")
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../../../lib/modules/generic/assertions.nix
    # END generated imports
  ];
}
