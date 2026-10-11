# Dependencies (callPackage)
{
  lib,
  stdenvNoCC,
  nix-module-imports,
}:

# testers.moduleImports function
# Docs: doc/build-helpers/testers.chapter.md
# Tests: ./tests.nix
{
  name,
  root,
  evaluations,
  definitionsOnly ? false,
  searchSeconds ? null,
}:
let
  # The files in the dependencies are named as the evaluations saw them,
  # and that name of `root` is not the name of the copy of `root` that the
  # build gets.
  evaluatedRoot = builtins.unsafeDiscardStringContext (toString root);

  dependencies = lib.imap0 (
    index: evaluation:
    builtins.toFile "dependencies-${toString index}.json" (
      builtins.unsafeDiscardStringContext (builtins.toJSON (lib.modules.moduleDependencies evaluation))
    )
  ) evaluations;
in
stdenvNoCC.mkDerivation {
  __structuredAttrs = true;
  strictDeps = true;
  name = "module-imports-${name}";
  src = root;
  dontUnpack = true; # The source is a tree that is read in place.
  nativeBuildInputs = [ nix-module-imports ];
  inherit dependencies evaluatedRoot;
  flags =
    lib.optional definitionsOnly "--definitions-only"
    ++ lib.optionals (searchSeconds != null) [
      "--search-seconds"
      (toString searchSeconds)
    ];
  doCheck = true;
  dontConfigure = true;
  dontBuild = true;
  checkPhase = ''
    arguments=(--root "$src" --evaluated-root "$evaluatedRoot" "''${flags[@]}")
    for file in "''${dependencies[@]}"; do
      arguments+=(--dependencies "$file")
    done
    nix-module-imports check "''${arguments[@]}"
  '';
  installPhase = ''
    touch "$out"
  '';
}
