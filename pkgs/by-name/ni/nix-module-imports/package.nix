{
  lib,
  rustPlatform,
  clippy,
  rustfmt,
  tests,
}:

let
  cargoToml = fromTOML (builtins.readFile ./Cargo.toml);
in
rustPlatform.buildRustPackage (finalAttrs: {
  pname = cargoToml.package.name;
  inherit (cargoToml.package) version;

  __structuredAttrs = true;

  src = lib.sourceFilesBySuffices ./. [
    ".rs"
    ".toml"
    ".lock"
  ];

  cargoLock = {
    lockFile = ./Cargo.lock;
  };

  passthru.tests = {
    lint-format = finalAttrs.finalPackage.overrideAttrs (
      _: previousAttrs: {
        pname = previousAttrs.pname + "-lint-format";
        nativeCheckInputs = (previousAttrs.nativeCheckInputs or [ ]) ++ [
          clippy
          rustfmt
        ];
        checkPhase = ''
          cargo clippy --all-targets -- -D warnings
          cargo fmt --check
        '';
      }
    );
    tester = tests.testers.moduleImports;
  };

  meta = {
    description = "Find, write and check the imports that the modules of a tree need";
    longDescription = ''
      A module of the module system in `lib.modules` works on its own
      only when it imports the modules whose options it uses. This tool
      takes the output of `lib.modules.moduleDependencies` for an
      evaluation of the modules and writes the missing imports into the
      module files, between two comments. It reports the files it
      cannot rewrite, and it can check a tree without changing it.
    '';
    license = lib.licenses.mit;
    maintainers = [ ];
    mainProgram = "nix-module-imports";
    platforms = lib.platforms.unix;
  };
})
