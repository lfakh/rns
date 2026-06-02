{ pkgs ? import <nixpkgs> {} }:

let
  # Libraries required at both build time and runtime
  libraries = with pkgs; [
    at-spi2-atk
    atkmm
    cairo
    gdk-pixbuf
    glib
    gtk3
    harfbuzz
    librsvg
    libsoup_3
    pango
    webkitgtk_4_1
    openssl
    libopus
    libayatana-appindicator
    dbus
  ];

  # Build tools
  packages = with pkgs; [
    curl
    wget
    pkg-config
    cmake
    ninja
    rustc
    cargo
    nodejs
    python3
  ] ++ libraries;
in
pkgs.mkShell {
  buildInputs = packages;

  # Set environment variables for Tauri, Gtk, and Opus
  shellHook = ''
    export LD_LIBRARY_PATH=${pkgs.lib.makeLibraryPath libraries}:$LD_LIBRARY_PATH
    
    # Comprehensive PKG_CONFIG_PATH for all Tauri/Opus dependencies
    export PKG_CONFIG_PATH="${pkgs.libopus.dev}/lib/pkgconfig:${pkgs.openssl.dev}/lib/pkgconfig:${pkgs.glib.dev}/lib/pkgconfig:${pkgs.gtk3.dev}/lib/pkgconfig:${pkgs.webkitgtk_4_1.dev}/lib/pkgconfig:${pkgs.pango.dev}/lib/pkgconfig:${pkgs.cairo.dev}/lib/pkgconfig:${pkgs.gdk-pixbuf.dev}/lib/pkgconfig:${pkgs.atk.dev}/lib/pkgconfig"
    
    # Fixes GSettings errors
    export XDG_DATA_DIRS=${pkgs.gsettings-desktop-schemas}/share/gsettings-schemas/${pkgs.gsettings-desktop-schemas.name}:${pkgs.gtk3}/share/gsettings-schemas/${pkgs.gtk3.name}:$XDG_DATA_DIRS
    
    echo "Nix Shell (V3): Heavyweight dev environment loaded."
  '';
}
