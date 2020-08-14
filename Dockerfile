FROM rust:1.43 as builder

RUN USER=root cargo new --bin captcha_service
WORKDIR ./captcha_service
COPY ./image.toml $HOME/.cargo/config
COPY ./Cargo.toml ./Cargo.toml
RUN cargo build --release
RUN rm src/*.rs

ADD . ./

RUN rm ./target/release/deps/captcha_service*
RUN cargo build --release

RUN cp ./target/release/captcha_service .

RUN ls

ENTRYPOINT ["./captcha_service"]
#
#FROM debian:buster-slim
#ARG APP=/usr/src/app
#
#RUN sed -i 's#http://deb.debian.org#https://mirrors.163.com#g' /etc/apt/sources.list
#RUN apt-get install apt-transport-https
#RUN apt-get update \
#    && apt-get install -y ca-certificates tzdata \
#    && rm -rf /var/lib/apt/lists/*
#
#EXPOSE 8088
#
#ENV TZ=Etc/UTC \
#    APP_USER=appuser
#
#RUN groupadd $APP_USER \
#    && useradd -g $APP_USER $APP_USER \
#    && mkdir -p ${APP}
#
#COPY --from=builder /captcha_service/target/release/captcha_service ${APP}/captcha_service
#
#RUN chown -R $APP_USER:$APP_USER ${APP}
#
#USER $APP_USER
#WORKDIR ${APP}
#
#CMD ["./captcha_service"]