RSYNCFLAGS := -avhs --delete
RSYNC = rsync $(RSYNCFLAGS)

SERVICE_STOP := systemctl --user stop
SERVICE_START := systemctl --user start
SERVICE_STATUS := systemctl --user status
SERVICE_JOURNAL := journalctl --user --lines 100 --follow --output short-precise --unit

SSHFLAGS := -t
SSH = ssh $(SSHFLAGS)

CARGOFLAGS :=
CARGO = cargo $(CARGOFLAGS)

CARGO_BUILDFLAGS :=
CARGO_BUILD = $(CARGO) build $(CARGO_BUILDFLAGS)

CARGO_RUNFLAGS :=
CARGO_RUN = $(CARGO) run $(CARGO_RUNFLAGS)

CURLFLAGS := -L
CURL = curl $(CURLFLAGS)
