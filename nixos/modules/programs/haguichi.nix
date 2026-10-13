{
  lib,
  pkgs,
  config,
  ...
}:

{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
    ../services/networking/logmein-hamachi.nix
    # END generated imports
  ];

  options.programs.haguichi = {
    enable = lib.mkEnableOption "Haguichi, a Linux GUI frontend to the proprietary LogMeIn Hamachi";
  };

  config = lib.mkIf config.programs.haguichi.enable {
    environment.systemPackages = with pkgs; [ haguichi ];

    services.logmein-hamachi.enable = true;
  };
}
