{ lib, ... }:

with lib;
{
  imports =
    warn
      ''
        `virtualisation.azure.agent` provided by `azure-agent.nix` module has been replaced
        by `services.waagent` options, and will be removed in a future release.
      ''
      [
        (mkRenamedOptionModule
          [
            "virtualisation"
            "azure"
            "agent"
            "enable"
          ]
          [
            "services"
            "waagent"
            "enable"
          ]
        )
        (mkRenamedOptionModule
          [
            "virtualisation"
            "azure"
            "agent"
            "verboseLogging"
          ]
          [
            "services"
            "waagent"
            "settings"
            "Logs"
            "Verbose"
          ]
        )
        (mkRenamedOptionModule
          [
            "virtualisation"
            "azure"
            "agent"
            "mountResourceDisk"
          ]
          [
            "services"
            "waagent"
            "settings"
            "ResourceDisk"
            "Format"
          ]
        )
      ];
}
