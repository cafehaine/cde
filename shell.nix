with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "cde";
    buildInputs = [ cargo rustfmt rustc clippy ]; # TODO switch to rustup?
}
