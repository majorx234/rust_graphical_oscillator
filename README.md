# Info
- generate sine wave with additional amplitude (AM) and frequency modulation (FM)
- possibility to change parameter
- visualize the wave
- learning signal processing 
- learning egui 

# build
```
cargo build
```

# run
```
cargo run
```

# ToDo 
- put wave generation in extra thread and copy data via ring buffer
- let paramter for samplerate be configured by jack settings
- add hull curve envelope and triggering via button
- add wav-file export
- add triggering via midi
- work with modules and reuseable code
- add polyphon control and wave generation

# History
- 20220720 jack sound output is working
- 20220712 first version with visualization of AM and FM 
