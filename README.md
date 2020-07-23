# Simple Alerts
The purpose of this application is to have a server listening for alert events to occur, and then to display those alert events on a page, showing the timestamps of when those alert events occurred.

Written in Rust and React.

## Admin Page
### Logging in
This is still a work in progress, but it works for now. To log in, from the admin page served at the base index of the application, open up your browser's developer console by hitting F12. Then, click the console tab, and enter: `document.cookie = 'ADMIN_TOKEN_HERE'`.

### CSRF Protection
There's no need to worry about CSRF attacks because all backend endpoints expect the `Authorization` header, and the javascript presented by the server to the client will read the cookie set above and will set make sure all javascript initiated URL requests to have an `Authorization` header with the value `Bearer COOKIE_SET_ABOVE`. Another site could not forward a user to one of these backend endpoints with a correct `Authorization` header because other sites cannot read the server site's cookie.

## Producing Alerts
An alert can be produced by making a `GET` web request to `https://ALERT_SERVER_FQDN:PORT/alerts/v1/write/ALERT_ID_HERE` using an Alerter token in the Authorization header as a Bearer token. Here's an example with curl's CLI:

```bash
curl https://ALERT_SERVER_FQDN:PORT/alerts/v1/write/MY_ALERT_ID_HERE  -H "Authorization: Bearer MY_ALERTER_TOKEN_here"
```

## Tokens
### Admin Tokens
These can be managed by making `GET` web requests to:
```
https://ALERT_SERVER_FQDN:PORT/admin/v1/list_admin_token
https://ALERT_SERVER_FQDN:PORT/admin/v1/write_admin_token
https://ALERT_SERVER_FQDN:PORT/admin/v1/delete_admin_token
```

These tokens are used to authorize all endpoints except the alerter write endpoint above. One should be used when setting the `document.cookie` variable, as mentioned above.

### Alerter Tokens
These can be managed by making `GET` web requests to:
```
https://ALERT_SERVER_FQDN:PORT/admin/v1/list_alerter_token
https://ALERT_SERVER_FQDN:PORT/admin/v1/write_alerter_token
https://ALERT_SERVER_FQDN:PORT/admin/v1/delete_alerter_token
```

These tokens are used to authorize the alert producing endpoint: `https://ALERT_SERVER_FQDN:PORT/alerts/v1/write/ALERT_ID_HERE`

## Docker image
The image can be found here: https://hub.docker.com/repository/docker/johnkordich/alerter/general

This container can be run like so:
```bash
docker run  \
  -e ROCKET_TLS='{certs="/simple_alerts/certstore/fullchain.pem",key="/simple_alerts/certstore/privkey.pem"}' \
  -e SUPER_ADMIN_TOKEN="MY_SUPER_ADMIN_TOKEN_HERE" \
  -v /home/johnkord/certstore:/simple_alerts/certstore \
  -p 443:8000 \
  johnkordich/alerter:latest
```

Notes about running the Docker container above:
- ROCKET_TLS contains paths to a cert chain and a private key, for running this server with TLS. If you don't want TLS, just remove that line. If you want to learn more about TLS and to this yourself, file an issue in this repository asking for help and I would be glad to explain how to set this up!
- The volume map (-v) option is for the certificate and private key. This should correlate to the ROCKET_TLS env var above it.
- The `SUPER_ADMIN_TOKEN` acts as an Admin Token, as explained above, but it cannot be removed by the `delete_admin_token` endpoint. You will need it to first log in.
- Set the port to any port you wish. The server internally listens on port 8000.
