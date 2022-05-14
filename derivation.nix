{ naersk, src, pkgs, lib, pkg-config, stops }:

naersk.buildPackage {
  pname = "telegram-decoder";
  version = "0.1.0";

  src = ./.;

  cargoSha256 = lib.fakeSha256;

  patchPhase = ''
    ls -alh
    cp ${stops}/stops.json ./stops.json
  '';

  nativeBuildInputs = with pkgs; [ pkg-config openssl];
  buildInputs = with pkgs; [ openssl ];

  meta = with lib; {
    description = "Sever which receives raw data and turns it into telegrams";
    homepage = "https://github.com/dump-dvb/telegram-decoder";
  };
}
