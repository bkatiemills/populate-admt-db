FROM rust:1.70.0 as base

RUN apt-get update -y ; apt-get install -y nano netcdf-bin libhdf5-serial-dev libnetcdff-dev

COPY nc2mongo /app
WORKDIR /app
RUN cargo build --release
RUN chown -R 1000660000 /app

FROM base as dev
# mockup for development
RUN mkdir /logs
RUN mkdir /logs/yesterday
RUN mkdir /logs/today
RUN mkdir -p /bulk/ifremer/aoml/1901727/profiles
RUN mkdir -p /bulk/ifremer/csiro/5903629/profiles
COPY devfiles/rsyncupdates /logs/today/.
COPY devfiles/R1901727_357.nc /bulk/ifremer/aoml/1901727/profiles/R1901727_357.nc
COPY devfiles/R1901727_358.nc /bulk/ifremer/aoml/1901727/profiles/R1901727_358.nc
COPY devfiles/BD5903629_098.nc /bulk/ifremer/csiro/5903629/profiles/BD5903629_098.nc
COPY devfiles/BD5903629_099.nc /bulk/ifremer/csiro/5903629/profiles/BD5903629_099.nc
COPY load_all.sh /app/load_all.sh
COPY load_update.sh /app/load_update.sh

FROM base as rebuild
COPY load_all.sh /app/load_all.sh
CMD ["bash", "/app/load_all.sh"]

FROM base as update
COPY load_update.sh /app/load_update.sh
CMD ["bash", "/app/load_update.sh"]