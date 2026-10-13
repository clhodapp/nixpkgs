{
  config,
  lib,
  pkgs,
  ...
}:
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/meta-maintainers.nix
    ../hardware/cpu/x86-msr.nix
    ../security/wrappers/default.nix
    # END generated imports
  ];

  options.programs.cpu-energy-meter = {
    enable = lib.mkEnableOption "CPU Energy Meter";
    package = lib.mkPackageOption pkgs "cpu-energy-meter" { };
  };

  config =
    let
      cfg = config.programs.cpu-energy-meter;
    in
    lib.mkIf cfg.enable {
      hardware.cpu.x86.msr.enable = true;

      security.wrappers.${cfg.package.meta.mainProgram} = {
        owner = "nobody";
        group = config.hardware.cpu.x86.msr.group;
        source = lib.getExe cfg.package;
        capabilities = "cap_sys_rawio=ep";
      };
    };

  meta.maintainers = with lib.maintainers; [ lorenzleutgeb ];
}
