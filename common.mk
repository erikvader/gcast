RSYNCFLAGS := -avhs --delete
RSYNC = rsync $(RSYNCFLAGS)

SERVICESTOP := systemctl --user stop
SERVICESTART := systemctl --user start
SERVICESTATUS := systemctl --user status

SSH := ssh
