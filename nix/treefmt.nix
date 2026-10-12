{
  rustfmtPackage,
  edition,
}: {...}: {
  projectRootFile = "flake.nix";

  programs.rustfmt = {
    enable = true;
    inherit edition;
    package = rustfmtPackage;
  };
  programs.alejandra.enable = true;
}
