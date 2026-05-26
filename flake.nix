{
  inputs.flake-utils.url = "github:numtide/flake-utils";
  inputs.fenix.url = "github:nix-community/fenix";
  inputs.fenix.inputs.nixpkgs.follows = "nixpkgs";

  outputs = {
    self,
    nixpkgs,
    flake-utils,
    fenix,
  }:
    flake-utils.lib.simpleFlake {
      inherit self nixpkgs;
      name = "nervana";
      overlay = fenix.overlays.default;
      shell = ./shell.nix;
    };
}
