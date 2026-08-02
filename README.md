# ibkr

Typed Rust client for the [Interactive Brokers Client Portal Web API](https://www.interactivebrokers.com/campus/ibkr-api-page/cpapi-v1/).

```toml
[dependencies]
ibkr = "0.4"
```

Two transports, one `Client`:

|  | Local gateway | First-party OAuth |
| --- | --- | --- |
| Host | `https://localhost:5000/v1/api` | `https://api.ibkr.com/v1/api` |
| Auth | browser login, repeated daily | live session token, valid ~24h |
| Requires | Client Portal Gateway running | registered consumer key |
| Suits | development | headless / unattended |

## Local gateway

Start the gateway and log in at <https://localhost:5000>:

```bash
bin/run.sh root/conf.yaml     # Windows: bin\run.bat root\conf.yaml
```

```rust
use ibkr::{iserver::secdef::search, Client};

let client = Client::new();
let hits = client.send(search::Request {
    symbol: "AAPL".into(),
    name: None,
    sec_type: None,
})?;
```

## OAuth

No gateway, no browser, no daily login. Register a consumer key and upload your public keys
per [IBKR's OAuth 1.0a guide](https://www.interactivebrokers.com/campus/ibkr-api-page/oauth-1-0a-extended/).

> **A newly registered consumer key does not work right away.** IBKR loads self-service OAuth
> keys during its daily server maintenance window — typically around midnight. Until that runs,
> minting a token fails. Register, then try again the next day.

```rust
use ibkr::{Client, Credentials};

let creds = Credentials::from_json(&json)?;  // file, stdin, secrets manager — your call
let lst = Client::mint(&creds)?;             // live session token, valid ~24h
let client = Client::oauth(&creds, &lst)?;
client.init_session()?;                      // open the brokerage session
```

### Credential payload

`Credentials::from_json` expects these keys. Generate the material, then assemble it however
your deployment prefers:

```bash
openssl genrsa -out private_signature.pem 2048
openssl rsa -in private_signature.pem -pubout -out public_signature.pem
openssl genrsa -out private_encryption.pem 2048
openssl rsa -in private_encryption.pem -pubout -out public_encryption.pem
openssl dhparam -out dhparam.pem 2048

# Upload both public_*.pem + dhparam.pem and generate the access token at
# https://ndcdyn.interactivebrokers.com/sso/Login?action=OAUTH&RL=1&ip2loc=US

read -rsp 'Access Token: '        ACCESS_TOKEN;        echo
read -rsp 'Access Token Secret: ' ACCESS_TOKEN_SECRET; echo
jq -n \
  --arg consumer_key        "$CONSUMER_KEY" \
  --arg realm               limited_poa \
  --arg access_token        "$ACCESS_TOKEN" \
  --arg access_token_secret "$ACCESS_TOKEN_SECRET" \
  --rawfile private_signature_pem  private_signature.pem \
  --rawfile private_encryption_pem private_encryption.pem \
  --rawfile dhparam_pem            dhparam.pem \
  '{consumer_key: $consumer_key, realm: $realm, access_token: $access_token,
    access_token_secret: $access_token_secret, private_signature_pem: $private_signature_pem,
    private_encryption_pem: $private_encryption_pem, dhparam_pem: $dhparam_pem}'
```

`realm` is `limited_poa`, or `test_realm` for IBKR's `TESTCONS` demo key.

Keep the result out of the repo — `*.pem` and `ibkr-secret.json` are gitignored. Piping it from a
secrets manager (AWS Secrets Manager, Vault, …) into your process avoids writing it to disk at all:

```bash
aws secretsmanager get-secret-value --secret-id ibkr/oauth/paper \
  --query SecretString --output text | cargo run --example search
```
