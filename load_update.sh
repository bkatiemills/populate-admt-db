# update the DB based on last night's rsync result
# expectations:
# - the rsync logs have been mounted at /logs
# - the rsync results have been mounted at /bulk/ifremer
# - the logs folder from last night is the most recent, and contains a file named rsyncupdates which lists every netcdf file touched by the rsync

updaterecord=$(ls -ltd -- /logs/ifremer/* | head -n 1 | awk '{print $NF}')
cat ${updaterecord}/rsyncupdates | sed 's|^|target/release/argo-data |' > update.sh
source update.sh