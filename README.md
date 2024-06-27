# populate-admt-db

Scripts and tools to generate a JSON representation of the full Argo dataset, and populate mongodb.

## rebuilding the database fron scratch

- generate empty argo and argoMeta collections with schema enforcement and indexes defined via this TBD process
- build the appropriate container target: `docker image build --target rebuild -t argovis/admtupdates:rebuild .`
- when running, make sure the results of rsync'ing ifremer are mounted at `/bulk/ifremer`; see `pod-rebuild.yaml` for example.

## updating nightly

- assumes that the most recently created subdirectory of `/logs` contains a file `rsyncresults` which lists the full path to every profile netCDF file CRUD'ed by the most recent rsync.
- build the appropriate container target: `docker image build --target update -t argovis/admtupdates:update .`
- when running, make sure the results of rsync'ing ifremer are mounted at `/bulk/ifremer` and the appropriate records are mounted at `/logs`; see `pod-update.yaml` for example.