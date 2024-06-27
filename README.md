# populate-admt-db

Scripts and tools to generate a JSON representation of the full Argo dataset, and populate mongodb.

## rebuilding the database fron scratch
- generate empty argo and argoMeta collections with schema enforcement and indexes defined via this TBD process
- build the appropriate container target: `docker image build --target rebuild -t argovis/admtupdates:rebuild .`
- when running, make sure the results of rsync'ing ifremer are mounted at `/bulk/ifremer`; see pod-rebuild.yaml for example.