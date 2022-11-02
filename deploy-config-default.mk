# The target triple given to rust
DEPLOY_TARGET := # It is probably this for a linux machine: x86_64-unknown-linux-gnu

# Username and ip to the machine to deploy to. Used by ssh and rsync
DEPLOY_HOST := # foo@192.168.1.10

# Where on the host the artifacts should go. These aren't configurable at the moment.
# The trailing slash is important for rsync.
DEPLOY_PATH := gcast
DEPLOY_CLIENT_PATH := $(DEPLOY_HOST):$(DEPLOY_PATH)/client/
DEPLOY_SERVER_PATH := $(DEPLOY_HOST):$(DEPLOY_PATH)/server/
