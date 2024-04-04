encode like this to get precise jumping: 
ffmpeg -i input.mp4 -force_key_frames "expr:gte(t,n_forced/60)" output.mp4


for file in *.mp4; do ffmpeg -i "$file" -force_key_frames "expr:gte(t,n_forced/60)" -y "${file%_.mp4}".mp4; done