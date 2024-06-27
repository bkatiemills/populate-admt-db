# rebuild the DB from scratch
# expectations:
# - the argo and argoMeta collections have been created with appropriate indexes and are empty
# - the rsync results have been mounted at /bulk/ifremer

find /bulk/ifremer/aoml -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > aoml.sh
bash aoml.sh
find /bulk/ifremer/bodc -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > bodc.sh
bash bodc.sh
find /bulk/ifremer/coriolis -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > coriolis.sh
bash coriolis.sh
find /bulk/ifremer/csio -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > csio.sh
bash csio.sh
find /bulk/ifremer/csiro -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > csiro.sh
bash csiro.sh
find /bulk/ifremer/incois -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > incois.sh
bash incois.sh
find /bulk/ifremer/jma -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > jma.sh
bash jma.sh
find /bulk/ifremer/kma -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > kma.sh
bash kma.sh
find /bulk/ifremer/kordi -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > kordi.sh
bash kordi.sh
find /bulk/ifremer/meds -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > meds.sh
bash meds.sh
find /bulk/ifremer/nmdis -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/convert_nc |' > nmdis.sh