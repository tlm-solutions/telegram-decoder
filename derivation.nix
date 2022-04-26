{ naersk, src, pkgs, lib, pkg-config }:

naersk.buildPackage {
  pname = "telegram-decoer";
  version = "0.1.0";

  src = ./.;

  cargoSha256 = lib.fakeSha256;

  nativeBuildInputs = with pkgs; [ pkg-config openssl];
  buildInputs = with pkgs; [ openssl ];

  meta = with lib; {
    description = "Sever which receives raw data and turns it into telegrams";
    homepage = "https://github.com/dump-dvb/telegram-decoder";
  };
}
