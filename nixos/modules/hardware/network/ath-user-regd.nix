{
  config,
  lib,
  pkgs,
  ...
}:
let
  kernelVersion = config.boot.kernelPackages.kernel.version;
  linuxKernelMinVersion = "5.8";
  kernelPatch = pkgs.kernelPatches.ath_regd_optional // {
    extraConfig = ''
      ATH_USER_REGD y
    '';
  };
in
{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/assertions.nix
    ../../system/boot/kernel.nix
    # END generated imports
  ];

  options.networking.wireless.athUserRegulatoryDomain = lib.mkOption {
    default = false;
    type = lib.types.bool;
    description = ''
      If enabled, sets the ATH_USER_REGD kernel config switch to true to
      disable the enforcement of EEPROM regulatory restrictions for ath
      drivers. Requires at least Linux ${linuxKernelMinVersion}.
    '';
  };

  config = lib.mkIf config.networking.wireless.athUserRegulatoryDomain {
    assertions = lib.singleton {
      assertion = lib.lessThan 0 (builtins.compareVersions kernelVersion linuxKernelMinVersion);
      message = "ATH_USER_REGD patch for kernels older than ${linuxKernelMinVersion} not ported yet!";
    };
    boot.kernelPatches = [ kernelPatch ];
  };
}
