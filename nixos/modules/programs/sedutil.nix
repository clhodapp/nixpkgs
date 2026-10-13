{
  config,
  lib,
  pkgs,
  ...
}:

let
  cfg = config.programs.sedutil;

in
{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../system/boot/kernel.nix
    # END generated imports
  ];

  options.programs.sedutil.enable = lib.mkEnableOption "sedutil, to manage self encrypting drives that conform to the Trusted Computing Group OPAL 2.0 SSC specification";

  config = lib.mkIf cfg.enable {
    boot.kernelParams = [
      "libata.allow_tpm=1"
    ];

    environment.systemPackages = with pkgs; [ sedutil ];
  };
}
