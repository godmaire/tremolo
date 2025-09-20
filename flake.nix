{
  inputs.nixpkgs.url = "github:NixOS/nixpkgs/nixos-unstable";

  outputs =
    { nixpkgs, ... }:
    let
      allSystems = [
        "x86_64-linux" # 64-bit Intel/AMD Linux
        "aarch64-linux" # 64-bit ARM Linux
        "x86_64-darwin" # 64-bit Intel macOS
        "aarch64-darwin" # 64-bit ARM macOS
      ];
      forAllSystems =
        f:
        nixpkgs.lib.genAttrs allSystems (
          system:
          f {
            inherit system;
            pkgs = import nixpkgs { inherit system; };
          }
        );
    in
    {
      devShell = forAllSystems (
        { pkgs, ... }:
        pkgs.mkShell {
          PKG_CONFIG_PATH = "${pkgs.openssl.dev}/lib/pkgconfig";
          DATABASE_URL = "postgres://postgres:password@localhost:5432/tremolo-db";
          TREMOLO_DATABASE_URL = "postgres://postgres:password@localhost:5432/tremolo-db";

          packages = with pkgs; [
            clang
            git
            openssl
            pkg-config

            bun
            just
            postgresql_17
            sqlx-cli
          ];
        }
      );
    };
}
