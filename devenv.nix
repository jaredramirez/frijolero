{ pkgs, lib, config, inputs, ... }:

let
  buildInputs = with pkgs; [ rust-bin.stable.latest.default just watchexec ];

  darwinBuildInputs = with pkgs;
    with pkgs.darwin;
    with pkgs.darwin.apple_sdk.frameworks; [
      libiconv
      Security
      AppKit
      CoreFoundation
      CoreAudio
      AudioToolbox
      AudioUnit
    ];
in {
  packages = buildInputs
    ++ lib.optionals pkgs.stdenv.isDarwin darwinBuildInputs;

  languages.rust.enable = true;

}
