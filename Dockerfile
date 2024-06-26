FROM rust:1.70.0

RUN apt-get update -y ; apt-get install -y nano netcdf-bin libhdf5-serial-dev libnetcdff-dev

COPY convert_nc /app
WORKDIR /app
RUN cargo build --release

# mockup for development
RUN mkdir /logs
RUN mkdir /logs/yesterday
RUN mkdir /logs/today
RUN mkdir -p /bulk/ifremer/aoml/1901727/profiles
COPY devfiles/rsyncupdates /logs/today/.
COPY devfiles/R1901727_357.nc /bulk/ifremer/aoml/1901727/profiles/R1901727_357.nc
COPY devfiles/R1901727_358.nc /bulk/ifremer/aoml/1901727/profiles/R1901727_358.nc
