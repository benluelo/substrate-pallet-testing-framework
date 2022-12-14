# Definitions can be imported from a separate file like this one
{self, ...}: {
  perSystem = {
    config,
    self',
    inputs',
    pkgs,
    ...
  }: {
    packages = {
      fmt = pkgs.writeShellApplication {
        name = "fmt";

        runtimeInputs = with pkgs; [
          alejandra
          nodePackages.prettier
          taplo
          cargo
        ];

        text = ''
          # .nix
          find . -name "*.nix" -type f -print0 | xargs -0 alejandra;
          # .toml
          taplo fmt --config="taplo.toml";
          # .rs
          cargo fmt;
          # .md
          prettier \
            --config=".prettierrc" \
            --write \
            --ignore-path=".prettierignore" \
            .
        '';
      };
    };
  };
}
