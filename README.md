# morning-compass-rust

RUSTFLAGS='-C target-feature=+crt-static' cargo build --release --target x86_64-unknown-linux-musl

# Default data
# ! MUST be inserted before actual usage !

! order must be maintained in db !

## ev_subscritptions

- FREE
- PLUS
- PRO
- ENTERPRISE

INSERT into ev_subscriptions ( subscription) VALUES ('FREE'), ('PLUS'), ('PRO'), ('ENTERPRISE');

## workspace_roles

- CREATOR
- ADMIN
- MANAGER
- WORKER

## roles

- USER // ev-user
- ADMIN // ev-admin
- SUPPORT // ev-support

INSERT into roles (name) VALUES ('USER'), ('ADMIN'), ('SUPPORT');
