{
  pkgs ? import <nixpkgs> { },
}:
pkgs.mkShell {
  name = "dwsh";

  nativeBuildInputs = with pkgs; [
    cargo
    rustc
    rust-analyzer
    clippy
    rustfmt
    eslint
    prettier

    pkg-config
  ];

  buildInputs = with pkgs; [
    pango
    gtk4
    gtk4-layer-shell
    librsvg
    dbus
  ];
}
