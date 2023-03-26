RSYNCFLAGS := -avhs --delete
RSYNC = rsync $(RSYNCFLAGS)

SERVICESTOP := systemctl --user stop
SERVICESTART := systemctl --user start
SERVICESTATUS := systemctl --user status
SERVICEJOURNAL := journalctl --user --lines 100 --follow --unit

SSH := ssh

CARGO := cargo
