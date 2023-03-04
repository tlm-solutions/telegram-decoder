{ config, lib, pkgs, ... }:
let
  cfg = config.TLMS.telegramDecoder;
in
{
  options.TLMS.telegramDecoder = with lib; {
    enable = mkOption {
      type = types.bool;
      default = false;
      description = ''Wether to enable TLMS telegram-decoder'';
    };
    server = mkOption {
      type = types.listOf types.str;
      default = [ "https://dump.dvb.solutions/" ];
      description = ''URL of the TLMS websocket'';
    };
    configFile = mkOption {
      type = types.either types.str types.path;
      default = "/etc/telegram-decoder/settings.json";
      description = ''Path to telegram-decoder config'';
    };
    authTokenFile = mkOption {
      type = types.either types.str types.path;
      default = "/etc/telegram-decoder/token";
      description = ''Path to telegram-decoder auth token'';
    };
    offline = mkOption {
      type = types.bool;
      default = false;
      description = ''runs telegram-decoder in offline mode used for mobile stations'';
    };
    logLevel = mkOption {
      type = types.enum [ "info" "warn" "error" "debug" "trace" ];
      default = "info";
      description = "under which logLevel the service should run";
    };
  };


  config = lib.mkIf cfg.enable {

    environment.systemPackages = [ pkgs.telegram-decoder ];

    users.groups.telegram-decoder = { };

    users.users.telegram-decoder = {
      name = "telegram-decoder";
      description = "gnu radio service user";
      group = "telegram-decoder";
      isSystemUser = true;
    };

    systemd.services."telegram-decoder" = {
      enable = true;
      wantedBy = [ "multi-user.target" ];

      script = "exec ${pkgs.telegram-decoder}/bin/telegram-decoder --config ${cfg.configFile} --server ${(builtins.concatStringsSep " " cfg.server)} ${if cfg.offline then "--offline" else ""}&";

      environment = {
        "RUST_LOG" = "${cfg.logLevel}";
        "AUTHENTICATION_TOKEN_PATH" = "${cfg.authTokenFile}";
      };

      serviceConfig = {
        Type = "forking";
        User = "telegram-decoder";
        Restart = "on-failure";
        StartLimitBurst = "2";
        StartLimitIntervalSec = "150s";
      };
    };
  };
}
