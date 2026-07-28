# ibkr

```bash
# 1. Start the Client Portal Gateway (from its directory), then log in at https://localhost:5000
bin/run.sh root/conf.yaml     # Windows: bin\run.bat root\conf.yaml

# 2. Fire the request
cargo run
```

## AWS Setup

For headless OAuth runs, store the IBKR credentials in AWS Secrets Manager.

```bash
# 1. Generate keys
openssl genrsa -out private_signature.pem 2048
openssl rsa -in private_signature.pem -outform PEM -pubout -out public_signature.pem
openssl genrsa -out private_encryption.pem 2048
openssl rsa -in private_encryption.pem -outform PEM -pubout -out public_encryption.pem
openssl dhparam -outform PEM -out dhparam.pem 2048

# 2. Upload the three public_*/dhparam files and generate the access token at:
#    https://ndcdyn.interactivebrokers.com/sso/Login?action=OAUTH&RL=1&ip2loc=US

# 3. Assemble the secret (paste tokens when prompted)
read -rsp 'Access Token: ' ACCESS_TOKEN; echo
read -rsp 'Access Token Secret: ' ACCESS_TOKEN_SECRET; echo
jq -n \
  --arg consumer_key        BJLACYPPR \
  --arg realm               limited_poa \
  --arg access_token        "$ACCESS_TOKEN" \
  --arg access_token_secret "$ACCESS_TOKEN_SECRET" \
  --rawfile private_signature_pem  private_signature.pem \
  --rawfile private_encryption_pem private_encryption.pem \
  --rawfile dhparam_pem            dhparam.pem \
  '{consumer_key: $consumer_key, realm: $realm, access_token: $access_token,
    access_token_secret: $access_token_secret, private_signature_pem: $private_signature_pem,
    private_encryption_pem: $private_encryption_pem, dhparam_pem: $dhparam_pem}' \
  > ibkr-secret.json

# 4. Store it, then delete the local copy
aws secretsmanager create-secret --name ibkr/oauth/paper --secret-string file://ibkr-secret.json
shred -u ibkr-secret.json
```
