{
  lib,
  rustPlatform,
  clippy,
  rustfmt,
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
  };

  meta = {
    description = "Keep the imports of module files in step with what the modules refer to";
    longDescription = ''
      A module of the module system in `lib.modules` sets and reads
      options. It works on its own only when it imports the modules that
      declare those options. This tool reads the source of the modules
      and finds, for each module, the options it refers to whose
      declaration it does not reach through its imports, and the paths
      it refers to that no module declares. `fix` writes the imports
      that are missing into the modules, between two comments, and
      `check` fails when a module does not have them. The tool
      evaluates nothing, so what it finds does not depend on a
      configuration.
    '';
    license = lib.licenses.mit;
    maintainers = [ ];
    mainProgram = "nix-module-imports";
    platforms = lib.platforms.unix;
  };
})
