{ self }:
{
  config,
  lib,
  pkgs,
  ...
}:
let
  cfg = config.programs.obs-studio.hermesix;
  system = pkgs.stdenv.hostPlatform.system;
in
{
  options.programs.obs-studio.hermesix = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = ''
        Whether to install Hermesix alongside Home Manager's OBS Studio module.
      '';
    };

    package = lib.mkOption {
      type = lib.types.package;
      default = self.packages.${system}.hermesix;
      defaultText = lib.literalExpression "inputs.hermesix.packages.${pkgs.stdenv.hostPlatform.system}.hermesix";
      description = ''
        Package providing Home Manager managed configuration tooling and OBS
        Studio export/sync compatibility commands.
      '';
    };
  };

  config = lib.mkIf (config.programs.obs-studio.enable && cfg.enable) {
    home.packages = [ cfg.package ];
  };
}
