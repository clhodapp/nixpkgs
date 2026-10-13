{
  config,
  pkgs,
  lib,
  ...
}:
let
  inherit (lib)
    mkEnableOption
    mkIf
    mkPackageOption
    ;

  cfg = config.programs.corectrl;
in
{
  imports = [
    (lib.mkRenamedOptionModule
      [ "programs" "corectrl" "gpuOverclock" "enable" ]
      [ "hardware" "amdgpu" "overdrive" "enable" ]
    )
    (lib.mkRenamedOptionModule
      [ "programs" "corectrl" "gpuOverclock" "ppfeaturemask" ]
      [ "hardware" "amdgpu" "overdrive" "ppfeaturemask" ]
    )
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../lib/modules/generic/assertions.nix
    ../../../lib/modules/generic/meta-maintainers.nix
    ../config/system-path.nix
    ../config/users-groups.nix
    ../security/polkit.nix
    ../services/hardware/amdgpu.nix
    ../services/system/dbus.nix
    # END generated imports
  ];

  options.programs.corectrl = {
    enable = mkEnableOption ''
      CoreCtrl, a tool to overclock amd graphics cards and processors.
      Add your user to the corectrl group to run corectrl without needing to enter your password
    '';

    package = mkPackageOption pkgs "corectrl" {
      extraDescription = "Useful for overriding the configuration options used for the package.";
    };
  };

  config = mkIf cfg.enable {
    environment.systemPackages = [ cfg.package ];

    services.dbus.packages = [ cfg.package ];

    users.groups.corectrl = { };

    security.polkit.extraConfig = ''
      polkit.addRule(function(action, subject) {
          if ((action.id == "org.corectrl.helper.init" ||
               action.id == "org.corectrl.helperkiller.init") &&
              subject.local == true &&
              subject.active == true &&
              subject.isInGroup("corectrl")) {
                  return polkit.Result.YES;
          }
      });
    '';
  };

  meta.maintainers = with lib.maintainers; [
    artturin
    Scrumplex
  ];
}
