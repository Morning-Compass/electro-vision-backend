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

## status (task status)

- HELP_NEEDED
- TODO
- IN_PROGRESS
- COMPLETED
- CANCELED

INSERT into status (name) VALUES ('HELP_NEEDED'), ('TODO'), ('IN_PROGRESS'), ('COMPLETED'), ('CANCELED');

INSERT into roles (name) VALUES ('USER'), ('ADMIN'), ('SUPPORT');

## importance

- LOW
- MEDIUM
- HIGH

INSERT into importance (name) VALUES ('LOW'), ('MEDIUM'), ('HIGH');

## task_category

- NONE, WORKSPACE_ID,
- ELECTRICAL, WORKSPACE_ID,
- VERIFICATION, WORKSPACE_ID,
- OTHER, WORKSPACE_ID,
