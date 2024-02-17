RSYNCFLAGS := -avhs --delete
RSYNC = rsync $(RSYNCFLAGS)

SERVICE_STOP := systemctl --user stop
SERVICE_START := systemctl --user start
SERVICE_STATUS := systemctl --user status
SERVICE_JOURNAL := journalctl --user --lines 100 --follow --unit

SSHFLAGS :=
SSH = ssh $(SSHFLAGS)

CARGOFLAGS :=
CARGO = cargo $(CARGOFLAGS)

CARGO_BUILDFLAGS :=
CARGO_BUILD = $(CARGO) build $(CARGO_BUILDFLAGS)

CURLFLAGS := -L
CURL = curl $(CURLFLAGS)
