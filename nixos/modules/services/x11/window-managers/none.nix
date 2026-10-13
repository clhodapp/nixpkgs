{
  imports = [
    # BEGIN generated imports: nix-module-imports writes the lines up to END
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
