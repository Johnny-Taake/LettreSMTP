# No-Reply-Email-Microservice

## Quickstart

1. Create `.env` file following `.env.example` or provide environment variables

2. Using Make run `make run` or using Cargo run `cargo run`, all Make and main docker commands are available in `Makefile`

3. Docs available at: </br>
Swagger: /docs </br>
OpenAPI: /openapi.json

4. Test endpoints

## Notes

- Simple per-IP rate limiting (per hour). For production scale, consider an external store.
- Email sending is offloaded from the async reactor to avoid blocking.
- File log `email_sent.log` stores minimal metadata (recipient masked in app logs; file log omits body).
