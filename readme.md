encode like this to get precise jumping: 
ffmpeg -i input.mp4 -force_key_frames "expr:gte(t,n_forced/60)" output.mp4
