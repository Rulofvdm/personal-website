{
  description = "Angular dev shell";

  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable"; # or your preferred channel
  };

  outputs = { self, nixpkgs, ... }:
    let
      system = "x86_64-linux"; # change if needed
      pkgs = import nixpkgs { inherit system; };
    in {
      devShells.${system}.default = pkgs.mkShell {
        packages = with pkgs; [
          nodejs_22
          angular-cli
          typescript
          nodePackages.npm
        ];

        shellHook = ''
          echo "Angular dev shell loaded"
          node --version
          ng version
        '';
      };
    };
}