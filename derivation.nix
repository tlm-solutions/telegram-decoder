{ naersk
, src
, lib
, pkg-config
, openssl
, cmake
, postgresql_14
}:

naersk.buildPackage {
  pname = "telegram-decoder";
  version = "0.3.0";

  src = ./.;

  cargoSha256 = lib.fakeSha256;

  nativeBuildInputs = [ pkg-config ];
  buildInputs = [ openssl cmake postgresql_14 ];

  meta = {
    description = "Sever which receives raw data and turns it into telegrams";
    homepage = "https://github.com/dump-dvb/telegram-decoder";
  };
}
