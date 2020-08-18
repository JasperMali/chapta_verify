FROM alpine:latest

COPY ./ /app
WORKDIR /app

RUN sed -i 's/dl-cdn.alpinelinux.org/mirrors.ustc.edu.cn/g' /etc/apk/repositories
RUN mkdir $HOME/.cargo
#COPY ./image.toml $HOME/.cargo/config
#	&& mv config $HOME/.cargo/
RUN apk add --no-cache libgcc
RUN apk add --no-cache --virtual .build-rust rust cargo
RUN cargo build --release
RUN cp target/release/captcha_service .
RUN rm -rf target/ ~/.cargo/
RUN apk del --purge .build-rust
RUN mkdir -p /data/captcha_service/

EXPOSE 8088

ENTRYPOINT ["./captcha_service"]