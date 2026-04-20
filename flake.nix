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
        ];

        shellHook = ''
          export NPM_CONFIG_PREFIX="$HOME/.npm-global"
          export PATH="$HOME/.npm-global/bin:$PATH"
          if ! command -v ng &> /dev/null; then
            echo "Installing @angular/cli..."
            npm install -g @angular/cli
          fi
          echo "Angular dev shell loaded — node $(node --version)"
        '';
      };
    };
}