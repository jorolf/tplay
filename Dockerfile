FROM rust:1.81

RUN apt update
RUN apt install -y build-essential ffmpeg libopencv-dev clang libclang-dev
RUN apt install -y librust-alsa-sys-dev
RUN apt install -y libavutil-dev libavformat-dev libavfilter-dev libavdevice-dev

RUN apt install -y python3-pip
RUN pip install yt-dlp --break-system-packages

WORKDIR /usr/src/tplay
COPY . .

RUN cargo install --path .

CMD ["tplay"]
