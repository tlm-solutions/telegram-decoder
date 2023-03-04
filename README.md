# Telegram Decoder ![](https://img.shields.io/endpoint?url=https%3A%2F%2Fhydra.hq.c3d2.de%2Fjob%2Fdvb-dump%2Fdecode-server%2Ftelegram-decoder.x86_64-linux%2Fshield)

![built with nix](https://builtwithnix.org/badge.svg)


**Contact:** <hello@tlm.solutions>

This service takes the raw bit stream from the gnuradio pipeline and tries to decode valid telegrams
from it. This service implement the full CRC and decodation of **R09.16** all other telegram variants
are sent as raw telegrams. All decoded telegram are sent to [data-accumulator](https://github.com/tlm-solutions/data-accumulator).


## Service Configuration 


### Environment Variables

- **AUTHENTICATION_TOKEN_PATH** per default set to `/etc/telegram-decoder/token`

### Config File

the config flag in the command line inputs points to a file following the pattern described below.

```
{
    "name": "Station Name",
    "lat": -1.0,
    "lon": -1.0,
    "id": "<YOUR STATION ID>",
    "region": -1
}
```

