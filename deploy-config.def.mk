# The target triple given to rust
DEPLOY_TARGET := # It is probably this for a linux machine: x86_64-unknown-linux-gnu

# Username and ip to the machine to deploy to. Used by ssh and rsync
DEPLOY_HOST := # foo@192.168.1.10

# Where on the host the artifacts should go
DEPLOY_PATH := #gcast
