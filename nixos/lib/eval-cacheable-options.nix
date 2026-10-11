{
  libPath,
  pkgsLibPath,
  nixosPath,
  modules,
  eager ? [ ],
  stateVersion,
  release,
}:

let
  lib = import libPath;
  modulesPath = "${nixosPath}/modules";
  # dummy pkgs set that contains no packages, only `pkgs.lib` from the full set.
  # not having `pkgs.lib` causes all users of `pkgs.formats` to fail.
  pkgs = import pkgsLibPath {
    inherit lib;
    pkgs = null;
  };
  utils = import "${nixosPath}/lib/utils.nix" {
    inherit config lib;
    pkgs = null;
  };
  # this is used both as a module and as specialArgs.
  # as a module it sets the _module special values, as specialArgs it makes `config`
  # unusable. this causes documentation attributes depending on `config` to fail.
  config = {
    _module.check = false;
    _module.args = { };
    system.stateVersion = stateVersion;
  };
  eval = lib.evalModules {
    modules = (map (m: "${modulesPath}/${m}") modules) ++ [
      config
    ];
    specialArgs = {
      inherit config pkgs utils;
      class = "nixos";
    };
  };
  docs = import "${nixosPath}/doc/manual" {
    pkgs = pkgs // {
      inherit lib;
      # duplicate of the declaration in all-packages.nix
      buildPackages.nixosOptionsDoc =
        attrs: (import "${nixosPath}/lib/make-options-doc") ({ inherit pkgs lib; } // attrs);
    };
    config = config.config;
    # A module in `modules` can import a module in `eager`. The options
    # of that module are documented outside the sandbox, and here they
    # are left out before they are rendered, because rendering them can
    # need module arguments that the sandbox does not have.
    options = lib.options.withoutOptionsDeclaredOnlyIn (map (
      m: "${modulesPath}/${m}"
    ) eager) eval.options;
    version = release;
    revision = "release-${release}";
    prefix = modulesPath;
    extraSources = [ (dirOf nixosPath) ];
  };
in
docs.optionsNix
