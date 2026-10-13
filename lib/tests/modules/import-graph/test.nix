let
  lib = import ../../..;

  # Two modules that import each other.
  first = {
    key = "first";
    imports = [ second ];
    options.first = lib.mkOption { default = "first"; };
  };
  second = {
    key = "second";
    imports = [ first ];
    options.second = lib.mkOption { default = "second"; };
  };

  cyclic = lib.evalModules { modules = [ first ]; };

  # A module that two others import.
  shared = {
    key = "shared";
    options.order = lib.mkOption { type = lib.types.listOf lib.types.str; };
    config.order = [ "shared" ];
  };

  # Layers of modules in which every module of a layer imports every
  # module of the next layer. Each module of the last layer is then
  # reached along `width` to the power of `depth - 1` paths, which is
  # ten to the power of nineteen here. The evaluation has to visit
  # each module once.
  depth = 20;
  width = 10;
  layer =
    level:
    builtins.genList (index: {
      key = "layer-${toString level}-module-${toString index}";
      imports = if level + 1 < depth then layer (level + 1) else [ ];
      options."layer${toString level}module${toString index}" = lib.mkOption { default = null; };
    }) width;

  layered = lib.evalModules { modules = layer 0; };

  # A module that a module in a cycle disables is left out, and the
  # cycle around it still evaluates.
  disabling = lib.evalModules {
    modules = [
      {
        key = "disabler";
        imports = [ first ];
        disabledModules = [ { key = "second"; } ];
      }
    ];
  };
in
assert
  cyclic.config == {
    first = "first";
    second = "second";
  };
assert builtins.length (builtins.attrNames layered.config) == depth * width;
# The graph of modules that import each other ends where a module comes
# up again below itself.
assert
  cyclic.graph == [
    {
      key = "first";
      file = "<unknown-file>";
      disabled = false;
      imports = [
        {
          key = "second";
          file = "<unknown-file>";
          disabled = false;
          imports = [
            {
              key = "first";
              file = "<unknown-file>";
              disabled = false;
              imports = [ ];
            }
          ];
        }
      ];
    }
  ];
# A module that two modules import is loaded once. The order of the
# definitions is the one that the module system gave before it visited
# each module once.
assert
  (lib.evalModules {
    modules = [
      {
        key = "left";
        imports = [ shared ];
        config.order = [ "left" ];
      }
      {
        key = "right";
        imports = [ shared ];
        config.order = [ "right" ];
      }
    ];
  }).config.order == [
    "shared"
    "right"
    "left"
  ];
assert
  builtins.attrNames disabling.options == [
    "_module"
    "first"
  ];
null
