# Info
- generate sine wave with additional amplitude (AM) and frequency modulation (FM)
- possibility to change parameter
- visualize the wave
- learning signal processing 
- learning egui 

# dependencies
You need JACK Audio Connection Kit and tools like qjackctl (see jackaudio.org)

On Ubuntu, you need to install dependencies:
```
sudo apt-get install jackd libjack-jackd2-dev
```

On Arch Linux:
```
sudo pacman -S jack2
```
and a newer version of cargo:
* tested with cargo 1.61.0 

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
- add hull curve envelope and triggering via button
- add wav-file export
- add triggering via midi
- work with modules and reuseable code
- add polyphon control and wave generation

# History
- 20220720 jack sound output is working
- 20220712 first version with visualization of AM and FM 

# Troubleshooting
* while build
```
stderr
  thread 'main' panicked at 'called `Result::unwrap()` on an `Err` value: "`\"pkg-config\" \"--libs\" \"--cflags\" \"jack\"` did not exit successfully: exit status: 1\nerror: could not find system library 'jack' required by the 'jack-sys' crate
  ...
  ```
  * got this on Ubuntu because `libjack-dev` or `libjack-jack2-dev` wasn't installed
  
  * with older version of cargo it couldn't found `eframe` > 0.17.0 but version 0.18.0 is required
