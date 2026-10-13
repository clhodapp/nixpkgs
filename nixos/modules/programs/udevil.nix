{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.udevil;

in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options.programs.udevil.enable = lib.mkEnableOption "udevil, to mount filesystems without password";

  config = lib.mkIf cfg.enable {
    security.wrappers.udevil = {
      setuid = true;
      owner = "root";
      group = "root";
      source = "${lib.getBin pkgs.udevil}/bin/udevil";
    };
  };
}
