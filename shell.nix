with import <nixpkgs> {};
stdenv.mkDerivation {
    name = "cde";
    buildInputs = [ cargo rustfmt rustc ]; # TODO switch to rustup?
}
