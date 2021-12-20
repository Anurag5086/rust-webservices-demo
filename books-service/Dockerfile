FROM rust:1.57
RUN mkdir /app
WORKDIR /app/
ADD . /app/
RUN cargo build
EXPOSE 8080
CMD ["cargo", "run"]