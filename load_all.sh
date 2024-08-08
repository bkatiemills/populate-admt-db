# rebuild the DB from scratch
# expectations:
# - the argo collection has been created with appropriate indexes and are empty
# - the rsync results have been mounted at /bulk/ifremer

find /bulk/ifremer/${1} -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > ${1}.sh
bash ${1}.sh

# find /bulk/ifremer/aoml -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > aoml.sh
# #bash aoml.sh
# find /bulk/ifremer/bodc -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > bodc.sh
# #bash bodc.sh
# find /bulk/ifremer/coriolis -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > coriolis.sh
# #bash coriolis.sh
# find /bulk/ifremer/csio -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > csio.sh
# #bash csio.sh
# find /bulk/ifremer/csiro -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > csiro.sh
# #bash csiro.sh
# find /bulk/ifremer/incois -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > incois.sh
# #bash incois.sh
# find /bulk/ifremer/jma -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > jma.sh
# #bash jma.sh
# find /bulk/ifremer/kma -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > kma.sh
# #bash kma.sh
# find /bulk/ifremer/kordi -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > kordi.sh
# #bash kordi.sh
# find /bulk/ifremer/meds -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > meds.sh
# #bash meds.sh
# find /bulk/ifremer/nmdis -type f | grep '/profiles/' | grep '.nc$' | sed 's|^|target/release/nc2mongo |' > nmdis.sh
# #bash nmdis.sh