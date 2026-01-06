{
  pkgs,
  lib,
  config,
  inputs,
  ...
}:
{
  # https://devenv.sh/basics/
  # env.GREET = "devenv";

  cachix.pull = [
    "m00nwtchr"
    "cargo2nix"
  ];

  # https://devenv.sh/packages/
  packages =
    with pkgs;
    [
      pkg-config
      openssl
      # mupdf deps
      freetype
      gumbo
      jbig2dec
      openjpeg
      libjpeg
      harfbuzz
      # build tools
      llvmPackages.clangUseLLVM # bindgen
    ]
    ++ lib.optionals (!config.container.isBuilding) [
      git
      cargo-nextest
    ];

  env.LIBCLANG_PATH = "${pkgs.llvmPackages.libclang.lib}/lib";

  languages.rust = {
    enable = true;
    mold.enable = true;
  };

  processes = {
    # cofdminer.exec = "cargo run";
  };

  treefmt = {
    enable = true;
    config.programs = {
      nixfmt.enable = true;
      rustfmt.enable = true;
    };
  };

  git-hooks.hooks = {
    treefmt.enable = true;
    clippy.enable = true;
  };

  tasks = {
    "cofd:tests" = lib.mkForce { };
    #   "cofdminer:tests" = {
    #     after = ["devenv:enterTest"];
    #     exec = "cargo nextest run";
    #   };
  };

  outputs = {
    # package Rust app using Nix
    # cofd-miner = config.languages.rust.import ./. {};
  };
  # See full reference at https://devenv.sh/reference/options/
}
