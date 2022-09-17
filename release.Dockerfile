FROM rust:1.62 AS builder
COPY . /CONTEXT
WORKDIR /CONTEXT
RUN cargo build --release
#RUN cargo install diesel_cli --version 1.4.1 --no-default-features --features mysql

#The context for "builder" was defined in the docker-compose file. /back in this case.
#The files in context are copied over to the build layer, filtered by .dockerignore.
#To access files needed to build and run the application we copy over the files brought into the context with the COPY command.
#The COPY command takes <src> <destination> as values. src is the context of the build layer. dest is some existing or new place in the layer.
#The WORKDIR changes the base path that commands such as COPY (<dest>) and RUN.
#Because we set WORKDIR, when we RUN cargo build it is from the CONTEXT folder with all the needed files.


FROM debian:buster-slim
#Must have the toml file or rocket will listen on localhost in the container.
COPY --from=builder /CONTEXT/Rocket.toml /Rocket.toml
COPY --from=builder /CONTEXT/target/release/back /back

#We pass in the values from the prior layer (defined as "builder" above) to our new layer. 

RUN ["apt", "update", "-y"]
RUN ["apt", "install", "libmariadb-dev-compat", "-y"]

#Unfortunately, a C library that diesel has as a dependency relies on a library found in most os, but not in buster-slim.
#https://stackoverflow.com/questions/62396248/why-rustc-did-not-include-libmariadb-into-release-binary