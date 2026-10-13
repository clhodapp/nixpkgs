{ lib, ... }:
{
  imports = [
    (lib.mkRemovedOptionModule [ "services" "github-runner" ] "Use `services.github-runners.*` instead")
    ./github-runner/options.nix
    ./github-runner/service.nix
    # BEGIN generated imports, written by `nix-module-imports fix`
    ../../../../lib/modules/generic/assertions.nix
    ../../../../lib/modules/generic/meta-maintainers.nix
    # END generated imports
  ];

  meta.maintainers = with lib.maintainers; [ veehaitch ];
}
