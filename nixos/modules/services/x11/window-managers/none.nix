{
  imports = [
    # BEGIN generated imports, written by `nix-module-imports fix`
    ./default.nix
    # END generated imports
  ];

  services = {
    xserver = {
      windowManager = {
        session = [
          {
            name = "none";
            start = "";
          }
        ];
      };
    };
  };
}
