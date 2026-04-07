{
  description = "Rust flake";
  inputs =
    {
      nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable"; # or whatever vers
      fenix.url = "github:nix-community/fenix/monthly";
    };

  outputs = { self, nixpkgs, fenix, ... }@inputs:
    let
      system = "x86_64-linux"; # your version
      pkgs = import nixpkgs {
        inherit system;
      };
      dlopenLibraries = with pkgs; [
        libxkbcommon

        # GPU backend
        vulkan-loader
        # libGL

        # Window system
        wayland
        libdbusmenu
        kdePackages.xdg-desktop-portal-kde
        libdisplay-info
        xorg.libX11
        xorg.libXcursor
        xorg.libXi
        xorg.libxcb
        zenity
      ];
      target = "thumbv8m.main-none-eabi";
      rust-toolchain = with fenix.packages.${system}; combine [
        minimal.rustc
        minimal.cargo
        minimal.rust-std
        default.rust-docs
        default.rustfmt
        default.clippy
        targets.x86_64-unknown-linux-gnu.latest.rust-std
        targets.${target}.latest.rust-std
      ];
    in
    {
      devShells.${system}.default = pkgs.mkShell
        {
          nativeBuildInputs = with pkgs; [ pkg-config cmake addDriverRunpath];
          buildInputs = [
            pkgs.udev
          ] ++ dlopenLibraries;
          packages = with pkgs; [
            minicom
            probe-rs-tools
            bacon
            rust-toolchain
            rust-analyzer
            uv
          ]; # whatever you need
          #env.RUSTFLAGS = "-C link-arg=-Wl,-rpath,${pkgs.lib.makeLibraryPath dlopenLibraries}";
          shellHook = ''
                 export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath (dlopenLibraries ++ [pkgs.udev])}:$LD_LIBRARY_PATH
            		 export LIBCLANG_PATH="${pkgs.llvmPackages.libclang.lib}/lib"
            		'';

        };
    };
}
