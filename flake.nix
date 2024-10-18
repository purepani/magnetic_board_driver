{
  description = "Rust flake";
  inputs =
    {
      nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; # or whatever vers
    };
  
  outputs = { self, nixpkgs, ... }@inputs:
    let
     system = "x86_64-linux"; # your version
     pkgs = nixpkgs.legacyPackages.${system};    
    in
    {
      devShells.${system}.default = pkgs.mkShell
      {
        packages = with pkgs; [ 
			rustup
			probe-rs
		]; # whatever you need
	
	shellHook = ''
		 export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
		'';

      };
    };
}
